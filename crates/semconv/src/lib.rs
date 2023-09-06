// SPDX-License-Identifier: Apache-2.0

//! This crate defines the concept of semantic convention catalog.

#![deny(missing_docs)]
#![deny(clippy::print_stdout)]

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::group::Group;

pub mod attribute;
pub mod group;

/// An error that can occur while loading a semantic convention catalog.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// The semantic convention asset was not found.
    #[error("Semantic convention catalog {path_or_url:?} not found\n{error:?}")]
    CatalogNotFound {
        /// The path or URL of the semantic convention asset.
        path_or_url: String,
        /// The error that occurred.
        error: String
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
    #[error("Duplicate atribute id `{id}` detected while loading {path_or_url:?}")]
    DuplicateAttributeId {
        /// The path or URL of the semantic convention asset.
        path_or_url: String,
        /// The duplicated attribute id.
        id: String,
    },

    /// The semantic convention asset contains an invalid attribute.
    #[error("Invalid attribute detected while resolving {path_or_url:?}, group_id=`{group_id}`.\n{error:?}")]
    InvalidAttribute {
        /// The path or URL of the semantic convention asset.
        path_or_url: String,
        /// The group id of the attribute.
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

    /// A collection of semantic convention attributes indexed by id.
    attributes: HashMap<String, attribute::Attribute>,
}

/// A semantic convention specification.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct SemConvSpec {
    /// A collection of semantic convention groups.
    pub groups: Vec<Group>,
}

impl SemConvCatalog {
    /// Load and add a semantic convention file to the catalog.
    pub fn load_from_file<P: AsRef<Path> + Clone>(&mut self, path: P) -> Result<(), Error> {
        let spec = SemConvSpec::load_from_file(path.clone())?;
        self.specs.push((path.as_ref().display().to_string(), spec));
        Ok(())
    }

    /// Load and add a semantic convention URL to the catalog.
    pub fn load_from_url(&mut self, semconv_url: &str) -> Result<(), Error> {
        let spec = SemConvSpec::load_from_url(semconv_url)?;
        self.specs.push((semconv_url.to_string(), spec));
        Ok(())
    }

    /// Resolves all the references present in the semantic convention catalog.
    pub fn resolve(&mut self) -> Result<(), Error> {
        struct AttributeToResolve {
            path_or_url: String,
            group_id: String,
            r#ref: String,
            attribute: attribute::Attribute,
        }

        let mut attributes_to_resolve = Vec::new();

        // Add all the attributes with an id to the catalog.
        for (path_or_url, spec) in self.specs.iter() {
            for group in spec.groups.iter() {
                let group_id = group.prefix.clone().unwrap_or_else(|| group.id.clone());

                match group.r#type {
                    // Resolve attributes
                    group::GroupType::AttributeGroup => {
                        for attr in group.attributes.iter() {
                            if let Some(id) = attr.id.as_ref() {
                                // The attribute has an id, so add it to the catalog
                                // if it does not exist yet, otherwise return an error.
                                // The fully qualified attribute id is the concatenation
                                // of the group id and the attribute id.
                                let fq_attr_id = format!("{}.{}", group_id, id);
                                let prev_val = self.attributes.insert(fq_attr_id.clone(), attr.clone());
                                if prev_val.is_some() {
                                    return Err(Error::DuplicateAttributeId {
                                        path_or_url: path_or_url.clone(),
                                        id: fq_attr_id.clone(),
                                    });
                                }
                            } else if let Some(r#ref) = attr.r#ref.as_ref() {
                                // The attribute has a reference, so add it to the
                                // list of attributes to resolve.
                                attributes_to_resolve.push(AttributeToResolve {
                                    path_or_url: path_or_url.clone(),
                                    group_id: group_id.clone(),
                                    r#ref: r#ref.clone(),
                                    attribute: attr.clone(),
                                });
                            } else {
                                // The attribute has neither an id nor a reference,
                                // so return an error.
                                return Err(Error::InvalidAttribute {
                                    path_or_url: path_or_url.clone(),
                                    group_id: group.id.clone(),
                                    error: "Attribute without id or ref".to_string(),
                                });
                            }
                        }
                    }
                    _ => {
                        println!("TODO: resolve other group types {:?}", group.r#type);
                    }
                }
            }
        }

        // Resolve all the attributes with a reference.
        for attr_to_resolve in attributes_to_resolve.into_iter() {
            let resolved_attr = self.attributes.get(&attr_to_resolve.r#ref);

            if let Some(resolved_attr) = resolved_attr {
                // Merge the resolved attribute with the attribute to resolve.
                let AttributeToResolve {mut attribute, ..} = attr_to_resolve;
                attribute.r#ref = None;
                attribute.id = resolved_attr.id.clone();
                self.attributes.insert(attr_to_resolve.r#ref, attribute);
            } else {
                return Err(Error::InvalidAttribute {
                    path_or_url: attr_to_resolve.path_or_url.clone(),
                    group_id: attr_to_resolve.group_id.clone(),
                    error: format!("Attribute reference '{}' not found", attr_to_resolve.r#ref),
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
