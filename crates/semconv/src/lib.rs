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
    /// The semantic convention catalog was not found.
    #[error("Semantic convention catalog {path_or_url:?} not found\n{error:?}")]
    CatalogNotFound {
        /// The path or URL of the semantic convention catalog.
        path_or_url: String,
        /// The error that occurred.
        error: String
    },

    /// The semantic convention catalog is invalid.
    #[error("Invalid semantic convention catalog {path_or_url:?}\n{error:?}")]
    InvalidCatalog {
        /// The path or URL of the semantic convention catalog.
        path_or_url: String,
        /// The line where the error occurred.
        line: Option<usize>,
        /// The column where the error occurred.
        column: Option<usize>,
        /// The error that occurred.
        error: String,
    },

    /// The semantic convention catalog contains a duplicate group id.
    #[error("Duplicate group id `{group_id}` detected while loading semantic convention catalog {path_or_url:?}")]
    DuplicateGroupId {
        /// The path or URL of the semantic convention catalog.
        path_or_url: String,
        /// The duplicated group id.
        group_id: String,
    },
}

/// A semantic convention catalog is a collection of semantic convention
/// specifications indexed by group id.
#[derive(Default)]
pub struct SemConvCatalog {
    /// A collection of semantic convention group indexed by id.
    specs: HashMap<String, Group>,
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
        self.add_semantic_convention_spec(path.as_ref().display().to_string(), spec)
    }

    /// Load and add a semantic convention URL to the catalog.
    pub fn load_from_url(&mut self, semconv_url: &str) -> Result<(), Error> {
        let spec = SemConvSpec::load_from_url(semconv_url)?;
        self.add_semantic_convention_spec(semconv_url.to_string(), spec)
    }

    /// Add a semantic convention specification to the catalog.
    pub fn add_semantic_convention_spec(&mut self, path_or_url: String, spec: SemConvSpec) -> Result<(), Error> {
        for group in spec.groups {
            let group_id = group.id.clone();
            let prev_val = self.specs.insert(group_id.clone(), group);
            if prev_val.is_some() {
                return Err(Error::DuplicateGroupId {
                    path_or_url,
                    group_id,
                });
            }
        }

        Ok(())
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

        for yaml in yaml_files {
            let catalog = SemConvSpec::load_from_file(yaml);
            assert!(catalog.is_ok(), "{:#?}", catalog.err().unwrap());
        }
    }
}
