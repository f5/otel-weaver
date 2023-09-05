// SPDX-License-Identifier: Apache-2.0

//! A Rust library for loading and validating telemetry schemas.

#![deny(missing_docs)]
#![deny(clippy::print_stdout)]

use crate::schema_spec::SchemaSpec;
use crate::version_spec::VersionSpec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub mod event;
pub mod instrumentation_library;
pub mod link;
pub mod log;
pub mod logs_change;
pub mod logs_version;
pub mod metrics_change;
pub mod metrics_version;
pub mod multivariate_metrics;
pub mod resource;
pub mod resource_change;
pub mod resource_logs;
pub mod resource_metrics;
pub mod resource_spans;
pub mod resource_version;
pub mod schema_spec;
pub mod span;
pub mod spans_change;
pub mod spans_version;
pub mod univariate_metric;
pub mod version_spec;

/// An error that can occur while loading a telemetry schema.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// The telemetry schema was not found.
    #[error("Schema {path_or_url:?} not found\n{error:?}")]
    SchemaNotFound {
        /// The path or URL of the telemetry schema.
        path_or_url: String,
        /// The error that occurred.
        error: String
    },

    /// The telemetry schema is invalid.
    #[error("Invalid schema {path_or_url:?}\n{error:?}")]
    InvalidSchema {
        /// The path or URL of the telemetry schema.
        path_or_url: String,
        /// The line number where the error occurred.
        line: Option<usize>,
        /// The column number where the error occurred.
        column: Option<usize>,
        /// The error that occurred.
        error: String,
    },
}

/// A telemetry schema.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct TelemetrySchema {
    /// The version of the telemetry schema specification.
    pub file_format: String,
    /// The URL of the parent schema (optional).
    pub parent_schema_url: Option<String>,
    /// The URL of the current schema.
    pub schema_url: String,
    /// The semantic conventions that are imported by the current schema (optional).
    #[serde(default)]
    pub semantic_conventions: Vec<SemConvImport>,
    /// The schema specification for the current schema.
    pub schema: Option<SchemaSpec>,
    /// The versions and corresponding changes that can be applied to the current schema.
    #[serde(default)]
    pub versions: HashMap<String, VersionSpec>,
}

/// A semantic convention import.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct SemConvImport {
    /// The URL of the semantic convention.
    pub url: String,
}

impl TelemetrySchema {
    /// Loads a telemetry schema file and returns the schema.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<TelemetrySchema, Error> {
        let path_buf = path.as_ref().to_path_buf();

        // Load and deserialize the telemetry schema
        let schema_file = File::open(path).map_err(|e| Error::SchemaNotFound {
            path_or_url: path_buf.as_path().display().to_string(),
            error: e.to_string(),
        })?;
        let schema: TelemetrySchema = serde_yaml::from_reader(BufReader::new(schema_file))
            .map_err(|e| Error::InvalidSchema {
                path_or_url: path_buf.as_path().display().to_string(),
                line: e.location().map(|loc| loc.line()),
                column: e.location().map(|loc| loc.column()),
                error: e.to_string(),
            })?;
        Ok(schema)
    }

    /// Loads a telemetry schema from a URL and returns the schema.
    pub fn load_from_url(schema_url: &str) -> Result<TelemetrySchema, Error> {
        // Create a content reader from the schema URL
        let reader = ureq::get(schema_url)
            .call()
            .map_err(|e| Error::SchemaNotFound {
                path_or_url: schema_url.to_string(),
                error: e.to_string(),
            })?
            .into_reader();

        // Deserialize the telemetry schema from the content reader
        let schema: TelemetrySchema =
            serde_yaml::from_reader(reader).map_err(|e| Error::InvalidSchema {
                path_or_url: schema_url.to_string(),
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
        let schema = TelemetrySchema::load_from_file("../../data/app-telemetry-schema.yaml");
        assert!(schema.is_ok(), "{:#?}", schema.err().unwrap());
    }
}
