use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::schema_spec::SchemaSpec;
use crate::version_spec::VersionSpec;

pub mod schema_spec;
pub mod version_spec;
pub mod resource;
pub mod resource_metrics;
pub mod resource_logs;
pub mod resource_spans;
pub mod spans_version;
pub mod spans_change;
pub mod metrics_version;
pub mod metrics_change;
pub mod resource_version;
pub mod resource_change;
pub mod logs_version;
pub mod logs_change;
pub mod univariate_metric;
pub mod multivariate_metrics;
pub mod log;
pub mod span;
pub mod event;
pub mod link;
pub mod instrumentation_library;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("schema not found (path: {path:?}, error: {error:?})")]
    SchemaNotFound {
        path: PathBuf,
        error: String,
    },

    #[error("invalid schema (path: {path:?}, line: {line:?}, column: {column:?}, error: {error:?})")]
    InvalidSchema {
        path: PathBuf,
        line: Option<usize>,
        column: Option<usize>,
        error: String,
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct TelemetrySchema {
    pub file_format: String,
    pub parent_schema_url: Option<String>,
    pub schema_url: String,
    #[serde(default)]
    pub semantic_conventions: Vec<SemConvImport>,
    pub schema: Option<SchemaSpec>,
    #[serde(default)]
    pub versions: HashMap<String, VersionSpec>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct SemConvImport {
    url: String,
}

impl TelemetrySchema {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<TelemetrySchema, Error> {
        let path_buf = path.as_ref().to_path_buf();

        // Load and deserialize the telemetry schema
        let schema_file = File::open(path).map_err(|e| Error::SchemaNotFound {
            path: path_buf.clone(),
            error: e.to_string()
        })?;
        let schema: TelemetrySchema = serde_yaml::from_reader(BufReader::new(schema_file))
            .map_err(|e| Error::InvalidSchema {
                path: path_buf,
                line: e.location().map(|loc| loc.line()),
                column: e.location().map(|loc| loc.column()),
                error: e.to_string(),
            })?;
        Ok(schema)
    }
}

#[cfg(test)]
mod test {
    use crate::TelemetrySchema;

    #[test]
    fn load_root_schema() {
        let schema = TelemetrySchema::load_from_file("data/root-schema-1.21.0.yaml");
        assert!(schema.is_ok(), "{:#?}", schema.err().unwrap());
    }

    #[test]
    fn load_app_telemetry_schema() {
        let schema = TelemetrySchema::load_from_file("data/app-telemetry-schema.yaml");
        assert!(schema.is_ok(), "{:#?}", schema.err().unwrap());
    }
}