use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::group::Group;

pub mod group;
pub mod attribute;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("semantic convention catalog not found (path: {path:?}, error: {error:?})")]
    CatalogNotFound {
        path: PathBuf,
        error: String,
    },

    #[error("invalid semantic convention catalog (path: {path:?}, line: {line:?}, column: {column:?}, error: {error:?})")]
    InvalidCatalog {
        path: PathBuf,
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
            path: path_buf.clone(),
            error: e.to_string(),
        })?;
        let catalog: Catalog = serde_yaml::from_reader(BufReader::new(catalog_file))
            .map_err(|e| Error::InvalidCatalog {
                path: path_buf,
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