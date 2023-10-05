// SPDX-License-Identifier: Apache-2.0

//! A resolver that can be used to resolve telemetry schemas.

#![deny(missing_docs)]
#![deny(clippy::print_stdout)]
#![deny(clippy::print_stderr)]

use std::collections::HashSet;
use std::path::Path;

use regex::Regex;
use url::Url;

use logger::Logger;
use schema::attribute::{Attribute, from_semconv_attributes};
use schema::metric_group::Metric;
use schema::tags::merge_with_override;
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
    #[error("Failed to resolve the attribute '{r#ref}'")]
    FailToResolveAttribute {
        /// The reference to the attribute.
        r#ref: String,
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
                Self::resolve_attributes(metrics.attributes.as_mut(), &sem_conv_catalog, version_changes.metric_attribute_changes())?;
                for metric in metrics.metrics.iter_mut() {
                    if let UnivariateMetric::Ref { r#ref, attributes, tags } = metric {
                        Self::resolve_attributes(attributes, &sem_conv_catalog, version_changes.metric_attribute_changes())?;
                        if let Some(referenced_metric) = sem_conv_catalog.get_metric(r#ref) {
                            let mut inherited_attrs = from_semconv_attributes(&referenced_metric.attributes);
                            Self::resolve_attributes(&mut inherited_attrs, &sem_conv_catalog, version_changes.metric_attribute_changes())?;
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
                    Self::resolve_attributes(metrics.attributes.as_mut(), &sem_conv_catalog, version_changes.metric_attribute_changes())?;
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
                Self::resolve_attributes(events.attributes.as_mut(), &sem_conv_catalog, version_changes.log_attribute_changes())?;
                for event in events.events.iter_mut() {
                    Self::resolve_attributes(event.attributes.as_mut(), &sem_conv_catalog, version_changes.log_attribute_changes())?;
                }
            }

            if let Some(spans) = schema.resource_spans.as_mut() {
                Self::resolve_attributes(spans.attributes.as_mut(), &sem_conv_catalog, version_changes.span_attribute_changes())?;
                for span in spans.spans.iter_mut() {
                    Self::resolve_attributes(span.attributes.as_mut(), &sem_conv_catalog, version_changes.span_attribute_changes())?;
                    for event in span.events.iter_mut() {
                        Self::resolve_attributes(event.attributes.as_mut(), &sem_conv_catalog, version_changes.span_attribute_changes())?;
                    }
                    for link in span.links.iter_mut() {
                        Self::resolve_attributes(link.attributes.as_mut(), &sem_conv_catalog, version_changes.span_attribute_changes())?;
                    }
                }
            }
            // Merge common attributes with the attributes of the corresponding resource_metrics,
            // resource_logs and resource_spans.
            if let Some(logs) = schema.resource_events.as_mut() {
                for log in logs.events.iter_mut() {
                    Self::resolve_attributes(log.attributes.as_mut(), &sem_conv_catalog, version_changes.resource_attribute_changes())?;
                }
            }
        }
        log.success(&format!(
            "Resolved schema '{}'",
            schema_path.as_ref().display()
        ));

        // Remove version information from the schema.
        schema.versions.take();

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

    fn resolve_attributes(
        attributes: &mut [Attribute],
        sem_conv_catalog: &semconv::SemConvCatalog,
        version_changes: impl VersionAttributeChanges) -> Result<(), Error> {
        for attribute in attributes.iter_mut() {
            if let Attribute::Ref {
                r#ref,
                brief: brief_from_ref,
                examples: examples_from_ref,
                tag: tag_from_ref,
                requirement_level: requirement_level_from_ref,
                sampling_relevant: sampling_from_ref,
                note: note_from_ref,
                stability: stability_from_ref,
                deprecated: deprecated_from_ref,
                tags: tags_from_ref,
                value: value_from_ref,
            } = attribute {
                let normalized_ref = version_changes.get_attribute_name(r#ref);
                if let Some(semconv::attribute::Attribute::Id {
                                id,
                                r#type,
                                brief,
                                examples,
                                tag,
                                requirement_level,
                                sampling_relevant,
                                note,
                                stability,
                                deprecated,
                            }) = sem_conv_catalog.get_attribute(&normalized_ref) {
                    let id = id.clone();
                    let r#type = r#type.clone();
                    let mut brief = brief.clone();
                    let mut examples = examples.clone();
                    let mut requirement_level = requirement_level.clone();
                    let mut tag = tag.clone();
                    let mut sampling_relevant = sampling_relevant.clone();
                    let mut note = note.clone();
                    let mut stability = stability.clone();
                    let mut deprecated = deprecated.clone();

                    // Override process.
                    // Use the field values from the reference when defined in the reference.
                    if let Some(brief_from_ref) = brief_from_ref {
                        brief = brief_from_ref.clone();
                    }
                    if let Some(requirement_level_from_ref) = requirement_level_from_ref {
                        requirement_level = requirement_level_from_ref.clone();
                    }
                    if let Some(examples_from_ref) = examples_from_ref {
                        examples = Some(examples_from_ref.clone());
                    }
                    if let Some(tag_from_ref) = tag_from_ref {
                        tag = Some(tag_from_ref.clone());
                    }
                    if let Some(sampling_from_ref) = sampling_from_ref {
                        sampling_relevant = Some(sampling_from_ref.clone());
                    }
                    if let Some(note_from_ref) = note_from_ref {
                        note = note_from_ref.clone();
                    }
                    if let Some(stability_from_ref) = stability_from_ref {
                        stability = Some(stability_from_ref.clone());
                    }
                    if let Some(deprecated_from_ref) = deprecated_from_ref {
                        deprecated = Some(deprecated_from_ref.clone());
                    }

                    *attribute = Attribute::Id {
                        id,
                        r#type,
                        brief,
                        examples,
                        tag,
                        requirement_level,
                        sampling_relevant,
                        note,
                        stability,
                        deprecated,
                        tags: tags_from_ref.clone(),
                        value: value_from_ref.clone()
                    }
                } else {
                    return Err(Error::FailToResolveAttribute {
                        r#ref: r#ref.clone(),
                    });
                }
            }
        }
        Ok(())
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
