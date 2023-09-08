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

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::group::Group;
use crate::metric::Metric;

pub mod attribute;
pub mod group;
pub mod metric;

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

    /// A collection of resolved attributes indexed by id.
    attributes: HashMap<String, attribute::Attribute>,

    /// A collection of resolved metrics indexed by id.
    metrics: HashMap<String, Metric>,
}

/// A semantic convention specification.
///
/// See [here](https://github.com/open-telemetry/build-tools/blob/main/semantic-conventions/syntax.md)
/// the syntax of the semantic convention YAML file.
#[derive(Serialize, Deserialize, Debug, Validate)]
#[serde(deny_unknown_fields)]
pub struct SemConvSpec {
    /// A collection of semantic convention groups.
    #[validate]
    pub groups: Vec<Group>,
}

struct AttributeToResolve {
    path_or_url: String,
    group_id: String,
    r#ref: String,
    attribute: attribute::Attribute,
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
    pub fn resolve(&mut self) -> Result<(), Error> {
        let mut attributes_to_resolve = Vec::new();
        let mut metrics_to_resolve = HashMap::new();

        // Add all the attributes with an id to the catalog.
        for (path_or_url, spec) in self.specs.iter() {
            for group in spec.groups.iter() {
                let group_id = if group.prefix.is_empty() {
                    group.id.clone()
                } else {
                    group.prefix.clone()
                };

                match group.r#type {
                    // Resolve attributes
                    group::ConvType::AttributeGroup => {
                        for attr in group.attributes.iter() {
                            if let Some(id) = attr.id.as_ref() {
                                // The attribute has an id, so add it to the catalog
                                // if it does not exist yet, otherwise return an error.
                                // The fully qualified attribute id is the concatenation
                                // of the group id and the attribute id.
                                let fq_attr_id = format!("{}.{}", group_id, id);
                                let mut attr_clone = attr.clone();
                                attr_clone.id = Some(fq_attr_id.clone());
                                let prev_val = self.attributes.insert(fq_attr_id.clone(), attr_clone);
                                if prev_val.is_some() {
                                    return Err(Error::DuplicateAttributeId {
                                        path_or_url: path_or_url.to_string(),
                                        id: fq_attr_id.clone(),
                                    });
                                }
                            } else if let Some(r#ref) = attr.r#ref.as_ref() {
                                // The attribute has a reference, so add it to the
                                // list of attributes to resolve.
                                attributes_to_resolve.push(AttributeToResolve {
                                    path_or_url: path_or_url.to_string(),
                                    group_id: group_id.to_string(),
                                    r#ref: r#ref.clone(),
                                    attribute: attr.clone(),
                                });
                            } else {
                                // The attribute has neither an id nor a reference,
                                // so return an error.
                                return Err(Error::InvalidAttribute {
                                    path_or_url: path_or_url.to_string(),
                                    group_id: group_id.to_string(),
                                    error: "Attribute without id or ref".to_string(),
                                });
                            }
                        }
                    }
                    group::ConvType::Span | group::ConvType::Resource => {
                        for attr in group.attributes.iter() {
                            if let Some(id) = attr.id.as_ref() {
                                // The attribute has an id, so add it to the catalog
                                // if it does not exist yet, otherwise return an error.
                                // The fully qualified attribute id is the concatenation
                                // of the group id and the attribute id.
                                let fq_attr_id = format!("{}.{}", group_id, id);
                                let mut attr_clone = attr.clone();
                                attr_clone.id = Some(fq_attr_id.clone());
                                let prev_val = self.attributes.insert(fq_attr_id.clone(), attr_clone);
                                if prev_val.is_some() {
                                    return Err(Error::DuplicateAttributeId {
                                        path_or_url: path_or_url.to_string(),
                                        id: fq_attr_id.clone(),
                                    });
                                }
                            } else if let Some(r#ref) = attr.r#ref.as_ref() {
                                // The attribute has a reference, so add it to the
                                // list of attributes to resolve.
                                attributes_to_resolve.push(AttributeToResolve {
                                    path_or_url: path_or_url.to_string(),
                                    group_id: group_id.to_string(),
                                    r#ref: r#ref.clone(),
                                    attribute: attr.clone(),
                                });
                            } else {
                                // The attribute has neither an id nor a reference,
                                // so return an error.
                                return Err(Error::InvalidAttribute {
                                    path_or_url: path_or_url.to_string(),
                                    group_id: group_id.to_string(),
                                    error: "Attribute without id or ref".to_string(),
                                });
                            }
                        }
                    }
                    group::ConvType::Metric => {
                        let metric_name = if let Some(metric_name) = group.metric_name.as_ref() {
                            metric_name.clone()
                        } else {
                            return Err(Error::InvalidMetric {
                                path_or_url: path_or_url.to_string(),
                                group_id: group_id.clone(),
                                error: "Metric without name".to_string(),
                            });
                        };

                        let prev_val = self.metrics.insert(metric_name.clone(), Metric {
                            name: metric_name.clone(),
                            brief: group.brief.clone(),
                            note: group.note.clone(),
                            attributes: vec![],
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
                    group::ConvType::Event => {
                        eprintln!("group type `event` not implemented yet`");
                    }
                    group::ConvType::MetricGroup => {
                        eprintln!("group type `metric_group` not implemented yet`");
                    }
                    group::ConvType::Scope => {
                        eprintln!("group type `scope` not implemented yet`");
                    }
                }
            }
        }

        // Resolve all the attributes with a reference.
        for attr_to_resolve in attributes_to_resolve.into_iter() {
            let resolved_attr = self.attributes.get(&attr_to_resolve.r#ref);

            if resolved_attr.is_none() {
                return Err(Error::InvalidAttribute {
                    path_or_url: attr_to_resolve.path_or_url.clone(),
                    group_id: attr_to_resolve.group_id.clone(),
                    error: format!("Attribute reference '{}' not found", attr_to_resolve.r#ref),
                });
            }
        }

        // Resolve all the metrics with an `extends` field.
        for (metric_name, r#ref) in metrics_to_resolve {
            let referenced_metric = self.metrics.get(&r#ref).cloned();
            if let Some(referenced_metric) = referenced_metric {
                let _ = self.metrics.get_mut(&metric_name).map(|metric| {
                    metric.attributes.extend(referenced_metric.attributes.iter().cloned());
                });
            }
        }

        self.specs.clear();

        Ok(())
    }

    /// Returns an attribute definition from its reference or `None` if the
    /// reference does not exist.
    pub fn get_attribute(&self, attr_ref: &str) -> Option<&attribute::Attribute> {
        self.attributes.get(attr_ref)
    }

    /// Returns a metric definition from its name or `None` if the
    /// name does not exist.
    pub fn get_metric(&self, metric_name: &str) -> Option<&Metric> {
        self.metrics.get(metric_name)
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

        let result = catalog.resolve();
        assert!(result.is_ok(), "{:#?}", result.err().unwrap());

        dbg!(catalog);
    }
}
