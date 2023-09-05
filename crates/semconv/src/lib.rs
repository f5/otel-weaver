//! This crate defines the concept of semantic convention catalog.

#![deny(missing_docs)]
#![deny(clippy::print_stdout)]

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
}

/// A semantic convention catalog.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    /// A collection of semantic convention groups.
    pub groups: Vec<Group>,
}

impl Catalog {
    /// Load a semantic convention catalog from a file.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Catalog, Error> {
        let path_buf = path.as_ref().to_path_buf();

        // Load and deserialize the semantic convention catalog
        let catalog_file = File::open(path).map_err(|e| Error::CatalogNotFound {
            path_or_url: path_buf.as_path().display().to_string(),
            error: e.to_string(),
        })?;
        let catalog: Catalog =
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
    pub fn load_from_url(semconv_url: &str) -> Result<Catalog, Error> {
        // Create a content reader from the semantic convention URL
        let reader = ureq::get(semconv_url)
            .call()
            .map_err(|e| Error::CatalogNotFound {
                path_or_url: semconv_url.to_string(),
                error: e.to_string(),
            })?
            .into_reader();

        // Deserialize the telemetry schema from the content reader
        let catalog: Catalog =
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
            let catalog = Catalog::load_from_file(yaml);
            assert!(catalog.is_ok(), "{:#?}", catalog.err().unwrap());
        }
    }
}
