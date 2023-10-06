// SPDX-License-Identifier: Apache-2.0

//! This crate defines the concept of a 'semantic convention catalog', which is
//! fueled by one or more semantic convention YAML files.
//!
//! The YAML language syntax used to define a semantic convention file
//! can be found [here](https://github.com/open-telemetry/build-tools/blob/main/semantic-conventions/syntax.md).

#![deny(
missing_docs,
clippy::print_stdout,
unstable_features,
unused_import_braces,
unused_qualifications,
unused_results,
unused_extern_crates,
)]

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::attribute::Attribute;
use crate::group::Group;
use crate::metric::Metric;

pub mod attribute;
pub mod group;
pub mod metric;
pub mod stability;

/// An error that can occur while loading a semantic convention catalog.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// The semantic convention asset was not found.
    #[error("Semantic convention catalog {path_or_url:?} not found\n{error:?}")]
    CatalogNotFound {
        /// The path or URL of the semantic convention asset.
        path_or_url: String,
        /// The error that occurred.
        error: String,
    },

    /// The semantic convention asset is invalid.
    #[error("Invalid semantic convention catalog {path_or_url:?}\n{error:?}")]
    InvalidCatalog {
        /// The path or URL of the semantic convention asset.
        path_or_url: String,
        /// The line where the error occurred.
        line: Option<usize>,
        /// The column where the error occurred.
        column: Option<usize>,
        /// The error that occurred.
        error: String,
    },

    /// The semantic convention asset contains a duplicate attribute id.
    #[error("Duplicate attribute id `{id}` detected while loading {path_or_url:?}")]
    DuplicateAttributeId {
        /// The path or URL of the semantic convention asset.
        path_or_url: String,
        /// The duplicated attribute id.
        id: String,
    },

    /// The semantic convention asset contains a duplicate group id.
    #[error("Duplicate group id `{id}` detected while loading {path_or_url:?} and already defined in {origin:?}")]
    DuplicateGroupId {
        /// The path or URL of the semantic convention asset.
        path_or_url: String,
        /// The duplicated group id.
        id: String,
        /// The asset where the group id was already defined.
        origin: String,
    },

    /// The semantic convention asset contains a duplicate metric name.
    #[error("Duplicate metric name `{name}` detected while loading {path_or_url:?}")]
    DuplicateMetricName {
        /// The path or URL of the semantic convention asset.
        path_or_url: String,
        /// The duplicated metric name.
        name: String,
    },

    /// The semantic convention asset contains an invalid attribute definition.
    #[error("Invalid attribute definition detected while resolving {path_or_url:?}, group_id=`{group_id}`.\n{error:?}")]
    InvalidAttribute {
        /// The path or URL of the semantic convention asset.
        path_or_url: String,
        /// The group id of the attribute.
        group_id: String,
        /// The reason of the error.
        error: String,
    },

    /// The attribute reference is not found.
    #[error("Attribute reference `{r#ref}` not found.")]
    AttributeNotFound {
        /// The attribute reference.
        r#ref: String,
    },

    /// The semantic convention asset contains an invalid metric definition.
    #[error("Invalid metric definition detected while resolving {path_or_url:?}, group_id=`{group_id}`.\n{error:?}")]
    InvalidMetric {
        /// The path or URL of the semantic convention asset.
        path_or_url: String,
        /// The group id of the metric.
        group_id: String,
        /// The reason of the error.
        error: String,
    },
}

/// A semantic convention catalog is a collection of semantic convention
/// specifications indexed by group id.
#[derive(Default, Debug)]
pub struct SemConvCatalog {
    /// A collection of semantic convention specifications loaded in the catalog.
    specs: Vec<(String, SemConvSpec)>,

    /// Attributes indexed by their respective id independently of their
    /// semantic convention group.
    ///
    /// This collection contains all the attributes defined in the catalog.
    all_attributes: HashMap<String, Attribute>,

    /// Metrics indexed by their respective id.
    ///
    /// This collection contains all the metrics defined in the catalog.
    all_metrics: HashMap<String, Metric>,

    /// Collection of attribute ids index by group id and defined in a
    /// `resource` semantic convention group.
    /// Attribute ids are references to of attributes defined in the
    /// all_attributes field.
    resource_group_attributes: HashMap<String, GroupIds>,

    /// Collection of attribute ids index by group id and defined in a
    /// `attribute_group` semantic convention group.
    /// Attribute ids are references to of attributes defined in the
    /// all_attributes field.
    attr_grp_group_attributes: HashMap<String, GroupIds>,

    /// Collection of attribute ids index by group id and defined in a
    /// `span` semantic convention group.
    /// Attribute ids are references to of attributes defined in the
    /// all_attributes field.
    span_group_attributes: HashMap<String, GroupIds>,

    /// Collection of attribute ids index by group id and defined in a
    /// `event` semantic convention group.
    /// Attribute ids are references to of attributes defined in the
    /// all_attributes field.
    event_group_attributes: HashMap<String, GroupIds>,

    /// Collection of attribute ids index by group id and defined in a
    /// `metric` semantic convention group.
    /// Attribute ids are references to of attributes defined in the
    /// all_attributes field.
    metric_group_attributes: HashMap<String, GroupIds>,

    /// Collection of attribute ids index by group id and defined in a
    /// `metric_group` semantic convention group.
    /// Attribute ids are references to of attributes defined in the
    /// all_attributes field.
    metric_group_group_attributes: HashMap<String, GroupIds>,
}

/// Represents a collection of ids (attribute or metric ids).
#[derive(Debug, Default)]
struct GroupIds {
    /// The semantic convention origin (path or URL) where the group id is
    /// defined. This is used to report errors.
    origin: String,
    /// The collection of ids (attribute or metric ids).
    ids: HashSet<String>,
}

/// A semantic convention specification.
///
/// See [here](https://github.com/open-telemetry/build-tools/blob/main/semantic-conventions/syntax.md)
/// the syntax of the semantic convention YAML file.
#[derive(Serialize, Deserialize, Debug, Validate, Clone)]
#[serde(deny_unknown_fields)]
pub struct SemConvSpec {
    /// A collection of semantic convention groups.
    #[validate]
    pub groups: Vec<Group>,
}

/// The configuration of the resolver.
#[derive(Debug, Default)]
pub struct ResolverConfig {
    error_when_attribute_ref_not_found: bool,
}

/// A wrapper for a resolver error that is considered as a warning
/// by configuration.
#[derive(Debug)]
pub struct ResolverWarning {
    /// The error that occurred.
    pub error: Error,
}

struct AttributeToResolve {
    path_or_url: String,
    group_id: String,
    r#ref: String,
}

impl SemConvCatalog {
    /// Load and add a semantic convention file to the catalog.
    pub fn load_from_file<P: AsRef<Path> + Clone>(&mut self, path: P) -> Result<(), Error> {
        let spec = SemConvSpec::load_from_file(path.clone())?;
        if let Err(e) = spec.validate() {
            return Err(Error::InvalidCatalog {
                path_or_url: path.as_ref().display().to_string(),
                line: None,
                column: None,
                error: e.to_string(),
            });
        }
        self.specs.push((path.as_ref().display().to_string(), spec));
        Ok(())
    }

    /// Load and add a semantic convention URL to the catalog.
    pub fn load_from_url(&mut self, semconv_url: &str) -> Result<(), Error> {
        let spec = SemConvSpec::load_from_url(semconv_url)?;
        if let Err(e) = spec.validate() {
            return Err(Error::InvalidCatalog {
                path_or_url: semconv_url.to_string(),
                line: None,
                column: None,
                error: e.to_string(),
            });
        }
        self.specs.push((semconv_url.to_string(), spec));
        Ok(())
    }

    /// Resolves all the references present in the semantic convention catalog.
    ///
    /// The `config` parameter allows to customize the resolver behavior
    /// when a reference is not found. By default, the resolver will emit an
    /// error when a reference is not found. This behavior can be changed by
    /// setting the `error_when_<...>_ref_not_found` to `false`, in which case
    /// the resolver will record the error in a warning list and continue.
    /// The warning list is returned as a list of warnings in the result.
    pub fn resolve(&mut self, config: ResolverConfig) -> Result<Vec<ResolverWarning>, Error> {
        let mut warnings = Vec::new();
        let mut attributes_to_resolve = Vec::new();
        let mut metrics_to_resolve = HashMap::new();

        // Add all the attributes with an id to the catalog.
        for (path_or_url, spec) in self.specs.clone().into_iter() {
            for group in spec.groups.iter() {
                // Process attributes
                match group.r#type {
                    group::ConvType::AttributeGroup | group::ConvType::Span
                    | group::ConvType::Resource | group::ConvType::Metric
                    | group::ConvType::Event | group::ConvType::MetricGroup => {
                        let attributes_in_group = self.process_attributes(
                            path_or_url.to_string(),
                            group.id.clone(),
                            group.prefix.clone(),
                            group.attributes.clone(),
                            &mut attributes_to_resolve,
                        )?;

                        let group_attributes = match group.r#type {
                            group::ConvType::AttributeGroup => { Some(&mut self.attr_grp_group_attributes) }
                            group::ConvType::Span => { Some(&mut self.span_group_attributes) }
                            group::ConvType::Resource => { Some(&mut self.resource_group_attributes) }
                            group::ConvType::Metric => { Some(&mut self.metric_group_attributes) }
                            group::ConvType::Event => { Some(&mut self.event_group_attributes) }
                            group::ConvType::MetricGroup => { Some(&mut self.metric_group_group_attributes) }
                            _ => { None }
                        };

                        if let Some(group_attributes) = group_attributes {
                            if !attributes_in_group.is_empty() {
                                Self::detect_duplicated_group(
                                    path_or_url.to_string(),
                                    group.id.clone(),
                                    group_attributes.insert(group.id.clone(), GroupIds {
                                        origin: path_or_url.to_string(),
                                        ids: attributes_in_group,
                                    }))?;
                            }
                        }
                    }
                    _ => {
                        eprintln!("Warning: group type `{:?}` not implemented yet", group.r#type);
                    }
                }

                // Process metrics
                match group.r#type {
                    group::ConvType::Metric => {
                        let metric_name = if let Some(metric_name) = group.metric_name.as_ref() {
                            metric_name.clone()
                        } else {
                            return Err(Error::InvalidMetric {
                                path_or_url: path_or_url.to_string(),
                                group_id: group.id.clone(),
                                error: "Metric without name".to_string(),
                            });
                        };

                        let prev_val = self.all_metrics.insert(metric_name.clone(), Metric {
                            name: metric_name.clone(),
                            brief: group.brief.clone(),
                            note: group.note.clone(),
                            attributes: group.attributes.clone(),
                            instrument: group.instrument.clone(),
                            unit: group.unit.clone(),
                        });
                        if prev_val.is_some() {
                            return Err(Error::DuplicateMetricName {
                                path_or_url: path_or_url.to_string(),
                                name: metric_name.clone(),
                            });
                        }

                        if let Some(r#ref) = group.extends.as_ref() {
                            let prev_val = metrics_to_resolve.insert(metric_name.clone(), r#ref.clone());
                            if prev_val.is_some() {
                                return Err(Error::DuplicateMetricName {
                                    path_or_url: path_or_url.to_string(),
                                    name: r#ref.clone(),
                                });
                            }
                        }
                    }
                    group::ConvType::MetricGroup => {
                        eprintln!("Warning: group type `metric_group` not implemented yet");
                    }
                    _ => {
                        // No metrics to process
                    }
                }
            }
        }

        // Resolve all the attributes with a reference.
        for attr_to_resolve in attributes_to_resolve.into_iter() {
            let resolved_attr = self.all_attributes.get(&attr_to_resolve.r#ref);

            if resolved_attr.is_none() {
                let err = Error::InvalidAttribute {
                    path_or_url: attr_to_resolve.path_or_url.clone(),
                    group_id: attr_to_resolve.group_id.clone(),
                    error: format!("Attribute reference '{}' not found", attr_to_resolve.r#ref),
                };
                if config.error_when_attribute_ref_not_found {
                    return Err(err);
                } else {
                    warnings.push(ResolverWarning {
                        error: err,
                    });
                }
            }
        }

        // Resolve all the metrics with an `extends` field.
        for (metric_name, r#ref) in metrics_to_resolve {
            let referenced_metric = self.all_metrics.get(&r#ref).cloned();
            if let Some(referenced_metric) = referenced_metric {
                let _ = self.all_metrics.get_mut(&metric_name).map(|metric| {
                    metric.attributes.extend(referenced_metric.attributes.iter().cloned());
                });
            }
        }

        self.specs.clear();

        Ok(warnings)
    }

    /// Returns an attribute definition from its reference or `None` if the
    /// reference does not exist.
    pub fn get_attribute(&self, attr_ref: &str) -> Option<&Attribute> {
        self.all_attributes.get(attr_ref)
    }

    /// Returns a map id -> attribute definition from an attribute group reference.
    /// Or an error if the reference does not exist.
    pub fn get_attributes(&self, r#ref: &str, r#type: group::ConvType) -> Result<HashMap<&String, &Attribute>, Error> {
        let mut attributes = HashMap::new();
        let group_ids = match r#type {
            group::ConvType::AttributeGroup => self.attr_grp_group_attributes.get(r#ref),
            group::ConvType::Span => self.span_group_attributes.get(r#ref),
            group::ConvType::Event => self.event_group_attributes.get(r#ref),
            group::ConvType::Metric => self.metric_group_attributes.get(r#ref),
            group::ConvType::MetricGroup => self.metric_group_group_attributes.get(r#ref),
            group::ConvType::Resource => self.resource_group_attributes.get(r#ref),
            group::ConvType::Scope => panic!("Scope not implemented yet"),
        };
        if let Some(group_ids) = group_ids {
            for attr_id in group_ids.ids.iter() {
                if let Some(attr) = self.all_attributes.get(attr_id) {
                    // Note: we only keep the last attribute definition for attributes that
                    // are defined multiple times in the group.
                    _ = attributes.insert(attr_id, attr);
                }
            }
        } else {
            return Err(Error::AttributeNotFound {
                r#ref: r#ref.to_string(),
            })
        }
        Ok(attributes)
    }

    /// Returns a metric definition from its name or `None` if the
    /// name does not exist.
    pub fn get_metric(&self, metric_name: &str) -> Option<&Metric> {
        self.all_metrics.get(metric_name)
    }

    /// Returns an error if prev_group_ids is not `None`.
    fn detect_duplicated_group(path_or_url: String, group_id: String, prev_group_ids: Option<GroupIds>) -> Result<(), Error> {
        if let Some(group_ids) = prev_group_ids.as_ref() {
            return Err(Error::DuplicateGroupId {
                path_or_url,
                id: group_id,
                origin: group_ids.origin.clone(),
            });
        }
        Ok(())
    }

    /// Processes a collection of attributes passed as a parameter (`attrs`),
    /// adds attributes fully defined to the catalog, adds attributes with
    /// a reference to the list of attributes to resolve and returns a
    /// collection of attribute ids defined in the current group.
    fn process_attributes(
        &mut self,
        path_or_url: String,
        group_id: String,
        prefix: String,
        attrs: Vec<Attribute>,
        attributes_to_resolve: &mut Vec<AttributeToResolve>,
    ) -> Result<HashSet<String>, Error> {
        let mut attributes_in_group = HashSet::new();
        for mut attr in attrs.into_iter() {
            match &attr {
                Attribute::Id { id, .. } => {
                    // The attribute has an id, so add it to the catalog
                    // if it does not exist yet, otherwise return an error.
                    // The fully qualified attribute id is the concatenation
                    // of the prefix and the attribute id (separated by a dot).
                    let fq_attr_id = if prefix.is_empty() {
                        id.clone()
                    } else {
                        format!("{}.{}", prefix, id)
                    };
                    if let Attribute::Id { id, .. } = &mut attr {
                        *id = fq_attr_id.clone();
                    }
                    let prev_val = self.all_attributes.insert(fq_attr_id.clone(), attr);
                    if prev_val.is_some() {
                        return Err(Error::DuplicateAttributeId {
                            path_or_url: path_or_url.clone(),
                            id: fq_attr_id.clone(),
                        });
                    }
                    let _ = attributes_in_group.insert(fq_attr_id.clone());
                }
                Attribute::Ref { r#ref, .. } => {
                    // The attribute has a reference, so add it to the
                    // list of attributes to resolve.
                    attributes_to_resolve.push(AttributeToResolve {
                        path_or_url: path_or_url.clone(),
                        group_id: group_id.clone(),
                        r#ref: r#ref.clone(),
                    });
                    let _ = attributes_in_group.insert(r#ref.clone());
                }
            }
        }
        Ok(attributes_in_group)
    }
}

impl SemConvSpec {
    /// Load a semantic convention catalog from a file.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<SemConvSpec, Error> {
        let path_buf = path.as_ref().to_path_buf();

        // Load and deserialize the semantic convention catalog
        let catalog_file = File::open(path).map_err(|e| Error::CatalogNotFound {
            path_or_url: path_buf.as_path().display().to_string(),
            error: e.to_string(),
        })?;
        let catalog: SemConvSpec =
            serde_yaml::from_reader(BufReader::new(catalog_file)).map_err(|e| {
                Error::InvalidCatalog {
                    path_or_url: path_buf.as_path().display().to_string(),
                    line: e.location().map(|loc| loc.line()),
                    column: e.location().map(|loc| loc.column()),
                    error: e.to_string(),
                }
            })?;
        Ok(catalog)
    }

    /// Load a semantic convention catalog from a URL.
    pub fn load_from_url(semconv_url: &str) -> Result<SemConvSpec, Error> {
        // Create a content reader from the semantic convention URL
        let reader = ureq::get(semconv_url)
            .call()
            .map_err(|e| Error::CatalogNotFound {
                path_or_url: semconv_url.to_string(),
                error: e.to_string(),
            })?
            .into_reader();

        // Deserialize the telemetry schema from the content reader
        let catalog: SemConvSpec =
            serde_yaml::from_reader(reader).map_err(|e| Error::InvalidCatalog {
                path_or_url: semconv_url.to_string(),
                line: e.location().map(|loc| loc.line()),
                column: e.location().map(|loc| loc.column()),
                error: e.to_string(),
            })?;
        Ok(catalog)
    }
}

#[cfg(test)]
mod tests {
    use std::{dbg, vec};

    use super::*;

    #[test]
    fn load_semconv_catalog() {
        let yaml_files = vec![
            "data/client.yaml",
            "data/cloud.yaml",
            "data/cloudevents.yaml",
            "data/database.yaml",
            "data/database-metrics.yaml",
            "data/exception.yaml",
            "data/faas.yaml",
            "data/faas-common.yaml",
            "data/faas-metrics.yaml",
            "data/http.yaml",
            "data/http-common.yaml",
            "data/http-metrics.yaml",
            "data/jvm-metrics.yaml",
            "data/media.yaml",
            "data/messaging.yaml",
            "data/network.yaml",
            "data/rpc.yaml",
            "data/rpc-metrics.yaml",
            "data/server.yaml",
            "data/source.yaml",
            "data/trace-exception.yaml",
            "data/url.yaml",
            "data/user-agent.yaml",
            "data/vm-metrics-experimental.yaml",
        ];

        let mut catalog = SemConvCatalog::default();
        for yaml in yaml_files {
            let result = catalog.load_from_file(yaml);
            assert!(result.is_ok(), "{:#?}", result.err().unwrap());
        }

        let result = catalog.resolve(ResolverConfig {
            error_when_attribute_ref_not_found: false,
        });

        dbg!(catalog);

        match result {
            Ok(warnings) => {
                if !warnings.is_empty() {
                    println!("warnings: {:#?}", warnings);
                }
                assert!(warnings.is_empty());
            }
            Err(e) => {
                panic!("{:#?}", e);
            }
        }
    }
}
