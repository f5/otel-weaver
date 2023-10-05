// SPDX-License-Identifier: Apache-2.0

//! A resolver that can be used to resolve telemetry schemas.

#![deny(missing_docs)]
#![deny(clippy::print_stdout)]
#![deny(clippy::print_stderr)]

use std::{clone, iter};
use std::collections::{HashMap, HashSet};
use std::path::Path;

use regex::Regex;
use url::Url;

use logger::Logger;
use schema::attribute::{Attribute, from_semconv_attributes};
use schema::metric_group::Metric;
pub use schema::TelemetrySchema;
use schema::univariate_metric::UnivariateMetric;
use semconv::ResolverConfig;
use version::{VersionAttributeChanges, VersionChanges};

/// A resolver that can be used to resolve telemetry schemas.
/// All references to semantic conventions will be resolved.
pub struct SchemaResolver {}

/// An error that can occur while resolving a telemetry schema.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// A telemetry schema error.
    #[error("Telemetry schema error (error: {0:?})")]
    TelemetrySchemaError(schema::Error),

    /// A parent schema error.
    #[error("Parent schema error (error: {0:?})")]
    ParentSchemaError(schema::Error),

    /// An invalid URL.
    #[error("Invalid URL `{url:?}`, error: {error:?})")]
    InvalidUrl {
        /// The invalid URL.
        url: String,
        /// The error that occurred.
        error: String,
    },

    /// A semantic convention error.
    #[error("Semantic convention error (error: {0:?})")]
    SemConvError(semconv::Error),

    /// Failed to resolve an attribute.
    #[error("Failed to resolve the attribute '{id}'")]
    FailToResolveAttribute {
        /// The id of the attribute.
        id: String,
        /// The error that occurred.
        error: String,
    },

    /// Failed to resolve a metric.
    #[error("Failed to resolve the metric '{r#ref}'")]
    FailToResolveMetric {
        /// The reference to the metric.
        r#ref: String,
    },
}

impl SchemaResolver {
    /// Loads a telemetry schema file and returns the resolved schema.
    pub fn resolve_schema_file<P: AsRef<Path> + Clone>(
        schema_path: P,
        log: &mut Logger,
    ) -> Result<TelemetrySchema, Error> {
        log.loading(&format!(
            "Loading schema '{}'",
            schema_path.as_ref().display()
        ));
        let mut schema = TelemetrySchema::load_from_file(schema_path.clone()).map_err(|e| {
            log.error(&format!(
                "Failed to load schema '{}'",
                schema_path.as_ref().display()
            ));
            Error::TelemetrySchemaError(e)
        })?;
        log.success(&format!(
            "Loaded schema '{}'",
            schema_path.as_ref().display()
        ));

        let parent_schema = Self::load_parent_schema(&schema, log)?;
        let mut sem_conv_catalog = Self::create_semantic_convention_catalog(&schema, log)?;
        let _ = sem_conv_catalog.resolve(ResolverConfig::default()).map_err(Error::SemConvError)?;

        // Merges the versions of the parent schema into the current schema.
        if let Some(parent_schema) = parent_schema {
            match schema.versions {
                Some(ref mut versions) => {
                    if let Some(parent_versions) = parent_schema.versions {
                        versions.extend(parent_versions);
                    }
                }
                None => {
                    schema.versions = parent_schema.versions;
                }
            }
        }

        // Generates version changes
        let version_changes = schema.versions.as_ref().map(|versions| if let Some(latest_version) = versions.latest_version() {
            versions.version_changes_for(latest_version)
        } else {
            VersionChanges::default()
        }).unwrap_or_default();

        // Resolve the references to the semantic conventions.
        log.loading("Solving semantic convention references");
        if let Some(schema) = schema.schema.as_mut() {
            // Resolve metrics and their attributes
            if let Some(metrics) = schema.resource_metrics.as_mut() {
                metrics.attributes = Self::resolve_attributes(metrics.attributes.as_ref(), &sem_conv_catalog, version_changes.metric_attribute_changes())?;
                for metric in metrics.metrics.iter_mut() {
                    if let UnivariateMetric::Ref { r#ref, attributes, tags } = metric {
                        *attributes = Self::resolve_attributes(attributes, &sem_conv_catalog, version_changes.metric_attribute_changes())?;
                        if let Some(referenced_metric) = sem_conv_catalog.get_metric(r#ref) {
                            let mut inherited_attrs = from_semconv_attributes(&referenced_metric.attributes);
                            inherited_attrs = Self::resolve_attributes(&inherited_attrs, &sem_conv_catalog, version_changes.metric_attribute_changes())?;
                            let merged_attrs = Self::merge_attributes(attributes, &inherited_attrs);
                            *metric = UnivariateMetric::Metric {
                                name: referenced_metric.name.clone(),
                                brief: referenced_metric.brief.clone(),
                                note: referenced_metric.note.clone(),
                                attributes: merged_attrs,
                                instrument: referenced_metric.instrument.clone(),
                                unit: referenced_metric.unit.clone(),
                                tags: tags.clone(),
                            };
                        } else {
                            return Err(Error::FailToResolveMetric {
                                r#ref: r#ref.clone(),
                            });
                        }
                    }
                }
                for metrics in metrics.metric_groups.iter_mut() {
                    metrics.attributes = Self::resolve_attributes(metrics.attributes.as_ref(), &sem_conv_catalog, version_changes.metric_attribute_changes())?;
                    for metric in metrics.metrics.iter_mut() {
                        if let Metric::Ref { r#ref, tags } = metric {
                            if let Some(referenced_metric) = sem_conv_catalog.get_metric(r#ref) {
                                let inherited_attrs = referenced_metric.attributes.clone();
                                if !inherited_attrs.is_empty() {
                                    log.warn(&format!("Attributes inherited from the '{}' metric will be disregarded. Instead, the common attributes specified for the metric group '{}' will be utilized.", r#ref, metrics.id));
                                }
                                *metric = Metric::Metric {
                                    name: referenced_metric.name.clone(),
                                    brief: referenced_metric.brief.clone(),
                                    note: referenced_metric.note.clone(),
                                    attributes: metrics.attributes.clone(),
                                    instrument: referenced_metric.instrument.clone(),
                                    unit: referenced_metric.unit.clone(),
                                    tags: tags.clone(),
                                };
                            } else {
                                return Err(Error::FailToResolveMetric {
                                    r#ref: r#ref.clone(),
                                });
                            }
                        }
                    }
                }
            }

            if let Some(events) = schema.resource_events.as_mut() {
                events.attributes = Self::resolve_attributes(events.attributes.as_ref(), &sem_conv_catalog, version_changes.log_attribute_changes())?;
                for event in events.events.iter_mut() {
                    event.attributes = Self::resolve_attributes(event.attributes.as_ref(), &sem_conv_catalog, version_changes.log_attribute_changes())?;
                }
            }

            if let Some(spans) = schema.resource_spans.as_mut() {
                spans.attributes = Self::resolve_attributes(spans.attributes.as_ref(), &sem_conv_catalog, version_changes.span_attribute_changes())?;
                for span in spans.spans.iter_mut() {
                    span.attributes = Self::resolve_attributes(span.attributes.as_ref(), &sem_conv_catalog, version_changes.span_attribute_changes())?;
                    for event in span.events.iter_mut() {
                        event.attributes = Self::resolve_attributes(event.attributes.as_ref(), &sem_conv_catalog, version_changes.span_attribute_changes())?;
                    }
                    for link in span.links.iter_mut() {
                        link.attributes = Self::resolve_attributes(link.attributes.as_ref(), &sem_conv_catalog, version_changes.span_attribute_changes())?;
                    }
                }
            }
            if let Some(events) = schema.resource_events.as_mut() {
                for event in events.events.iter_mut() {
                    event.attributes = Self::resolve_attributes(event.attributes.as_ref(), &sem_conv_catalog, version_changes.resource_attribute_changes())?;
                }
            }
        }
        log.success(&format!(
            "Resolved schema '{}'",
            schema_path.as_ref().display()
        ));

        schema.semantic_conventions.clear();

        Ok(schema)
    }

    /// Loads the parent telemetry schema if it exists.
    fn load_parent_schema(schema: &TelemetrySchema, log: &mut Logger) -> Result<Option<TelemetrySchema>, Error> {
        // Load the parent schema and merge it into the current schema.
        let parent_schema = if let Some(parent_schema_url) = schema.parent_schema_url.as_ref() {
            log.loading(&format!("Loading parent schema '{}'", parent_schema_url));
            let url_pattern = Regex::new(r"^(https|http|file):.*").expect("invalid regex, please report this bug");
            let parent_schema = if url_pattern.is_match(parent_schema_url) {
                let url = Url::parse(parent_schema_url).map_err(|e| {
                    log.error(&format!(
                        "Failed to parset parent schema url '{}'",
                        parent_schema_url
                    ));
                    Error::InvalidUrl {
                        url: parent_schema_url.clone(),
                        error: e.to_string(),
                    }
                })?;
                TelemetrySchema::load_from_url(&url).map_err(|e| {
                    log.error(&format!(
                        "Failed to load parent schema '{}'",
                        parent_schema_url
                    ));
                    Error::ParentSchemaError(e)
                })?
            } else {
                TelemetrySchema::load_from_file(parent_schema_url).map_err(|e| {
                    log.error(&format!(
                        "Failed to load parent schema '{}'",
                        parent_schema_url
                    ));
                    Error::ParentSchemaError(e)
                })?
            };

            log.success(&format!("Loaded parent schema '{}'", parent_schema_url));
            Some(parent_schema)
        } else {
            None
        };

        Ok(parent_schema)
    }

    /// Creates a semantic convention catalog from the given telemetry schema.
    fn create_semantic_convention_catalog(schema: &TelemetrySchema, log: &mut Logger) -> Result<semconv::SemConvCatalog, Error> {
        // Load all the semantic convention catalogs.
        let mut sem_conv_catalog = semconv::SemConvCatalog::default();
        log.loading(&format!(
            "Loading {} semantic convention catalogs",
            schema.semantic_conventions.len()
        ));
        for sem_conv_import in schema.semantic_conventions.iter() {
            sem_conv_catalog.load_from_url(&sem_conv_import.url).map_err(|e| {
                log.error(&e.to_string());
                Error::SemConvError(e)
            })?;
        }
        log.success(&format!(
            "Loaded {} semantic convention catalogs",
            schema.semantic_conventions.len()
        ));

        Ok(sem_conv_catalog)
    }

    /// Resolves a collection of attributes (i.e. `Attribute::Ref` and `Attribute::AttributeGroupRef`)
    /// from the given semantic convention catalog and local attributes (i.e. `Attribute::Id`).
    /// `Attribute::AttributeGroupRef` are first resolved, then `Attribute::Ref`, and finally
    /// `Attribute::Id` are added.
    /// An `Attribute::Ref` can override an attribute contains in an `Attribute::AttributeGroupRef`.
    /// An `Attribute::Id` can override an attribute contains in an `Attribute::Ref` or an
    /// `Attribute::AttributeGroupRef`.
    ///
    /// Note: Version changes are used during the resolution process to determine the names of the
    /// attributes.
    fn resolve_attributes(
        attributes: &[Attribute],
        sem_conv_catalog: &semconv::SemConvCatalog,
        version_changes: impl VersionAttributeChanges,
    ) -> Result<Vec<Attribute>, Error> {
        let mut resolved_attributes = HashMap::new();

        // Resolve `Attribute::AttributeGroupRef`
        for attribute in attributes.iter() {
            if let Attribute::AttributeGroupRef { attribute_group_ref, tags } = attribute {
                for (attr_id, attr) in sem_conv_catalog.get_attributes(&attribute_group_ref) {
                    let mut attr: Attribute = attr.into();
                    attr.set_tags(tags);
                    resolved_attributes.insert(attr_id.clone(), attr);
                }
            }
        }

        // Resolve `Attribute::Ref`
        for attribute in attributes.iter() {
            if let Attribute::Ref { r#ref, .. } = attribute {
                let normalized_ref = version_changes.get_attribute_name(r#ref);
                let sem_conv_attr = sem_conv_catalog.get_attribute(&normalized_ref);
                let resolved_attribute = attribute.resolve_from(sem_conv_attr).map_err(|e| Error::FailToResolveAttribute {
                    id: r#ref.clone(),
                    error: e.to_string(),
                })?;
                resolved_attributes.insert(normalized_ref, resolved_attribute);
            }
        }

        // Resolve `Attribute::Id`
        // Note: any resolved attributes with the same id will be overridden.
        for attribute in attributes.iter() {
            if let Attribute::Id { id, .. } = attribute {
                resolved_attributes.insert(id.clone(), attribute.clone());
            }
        }

        Ok(resolved_attributes.into_values().collect())
    }

    /// Merges the given main attributes with the inherited attributes.
    /// Main attributes have precedence over inherited attributes.
    fn merge_attributes(main_attrs: &[Attribute], inherited_attrs: &[Attribute]) -> Vec<Attribute> {
        let mut merged_attrs = main_attrs.to_vec();
        let main_attr_ids = main_attrs.iter().map(|attr| match attr {
            Attribute::Ref { r#ref, .. } => r#ref.clone(),
            Attribute::Id { id, .. } => id.clone(),
            Attribute::AttributeGroupRef { .. } => {
                panic!("Attribute groups are not supported yet")
            }
        }).collect::<HashSet<_>>();

        for inherited_attr in inherited_attrs.iter() {
            match inherited_attr {
                Attribute::Ref { r#ref, .. } => {
                    if main_attr_ids.contains(r#ref) {
                        continue;
                    }
                }
                Attribute::Id { id, .. } => {
                    if main_attr_ids.contains(id) {
                        continue;
                    }
                }
                Attribute::AttributeGroupRef { .. } => {
                    panic!("Attribute groups are not supported yet")
                }
            }
            merged_attrs.push(inherited_attr.clone());
        }
        merged_attrs
    }
}

#[cfg(test)]
mod test {
    use logger::Logger;

    use crate::SchemaResolver;

    #[test]
    fn resolve_schema() {
        let mut log = Logger::new(0);
        let schema =
            SchemaResolver::resolve_schema_file("../../data/app-telemetry-schema.yaml", &mut log);
        assert!(schema.is_ok(), "{:#?}", schema.err().unwrap());
    }
}
