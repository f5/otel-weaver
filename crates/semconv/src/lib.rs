use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::group::Group;

pub mod group;
pub mod attribute;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Semantic convention catalog {path_or_url:?} not found\n{error:?}")]
    CatalogNotFound {
        path_or_url: String,
        error: String,
    },

    #[error("Invalid semantic convention catalog {path_or_url:?}\n{error:?}")]
    InvalidCatalog {
        path_or_url: String,
        line: Option<usize>,
        column: Option<usize>,
        error: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub groups: Vec<Group>,
}

impl Catalog {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Catalog, Error> {
        let path_buf = path.as_ref().to_path_buf();

        // Load and deserialize the semantic convention catalog
        let catalog_file = File::open(path).map_err(|e| Error::CatalogNotFound {
            path_or_url: path_buf.as_path().display().to_string(),
            error: e.to_string(),
        })?;
        let catalog: Catalog = serde_yaml::from_reader(BufReader::new(catalog_file))
            .map_err(|e| Error::InvalidCatalog {
                path_or_url: path_buf.as_path().display().to_string(),
                line: e.location().map(|loc| loc.line()),
                column: e.location().map(|loc| loc.column()),
                error: e.to_string(),
            })?;
        Ok(catalog)
    }

    pub fn load_from_url(semconv_url: &str) -> Result<Catalog, Error> {
        // Create a content reader from the semantic convention URL
        let reader = ureq::get(semconv_url)
            .call().map_err(|e| Error::CatalogNotFound {
            path_or_url: semconv_url.to_string(),
            error: e.to_string(),
        })?
            .into_reader();

        // Deserialize the telemetry schema from the content reader
        let catalog: Catalog = serde_yaml::from_reader(reader)
            .map_err(|e| Error::InvalidCatalog {
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