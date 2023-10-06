// SPDX-License-Identifier: Apache-2.0

//! A Rust library for loading and validating telemetry schemas.

#![deny(missing_docs)]
#![deny(clippy::print_stdout)]
#![deny(clippy::print_stderr)]

use crate::schema_spec::SchemaSpec;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use url::Url;
use version::{Versions};

pub mod event;
pub mod instrumentation_library;
pub mod span_link;
pub mod span_event;
pub mod metric_group;
pub mod resource;
pub mod resource_events;
pub mod resource_metrics;
pub mod resource_spans;
pub mod schema_spec;
pub mod span;
pub mod univariate_metric;
pub mod attribute;
pub mod log;
pub mod tags;

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

    /// The attribute is invalid.
    #[error("Invalid attribute `{id:?}`\n{error:?}")]
    InvalidAttribute {
        /// The attribute id.
        id: String,
        /// The error that occurred.
        error: String,
    }
}

/// A telemetry schema.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct TelemetrySchema {
    /// Defines the file format. MUST be set to 1.2.0.
    pub file_format: String,
    /// Optional field specifying the schema url of the parent schema. The current
    /// schema overrides the parent schema.
    /// Usually the parent schema is the official OpenTelemetry Telemetry schema
    /// containing the versioning and their corresponding transformations.
    /// However, it can also include any of the new fields defined in this OTEP.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_schema_url: Option<String>,
    /// The Schema URL that this file is published at.
    pub schema_url: String,
    /// The semantic conventions that are imported by the current schema (optional).
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub semantic_conventions: Vec<SemConvImport>,
    /// Definition of the telemetry schema for an application or a library.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<SchemaSpec>,
    /// Definitions for each schema version in this family.
    /// Note: the ordering of versions is defined according to semver
    /// version number ordering rules.
    /// This section is described in more details in the OTEP 0152 and in a dedicated
    /// section below.
    /// https://github.com/open-telemetry/oteps/blob/main/text/0152-telemetry-schemas.md
    #[serde(skip_serializing_if = "Option::is_none")]
    pub versions: Option<Versions>,
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
    pub fn load_from_url(schema_url: &Url) -> Result<TelemetrySchema, Error> {
        match schema_url.scheme() {
            "http" | "https" => {
                // Create a content reader from the schema URL
                let reader = ureq::get(&schema_url.to_string())
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
            "file" => {
                let path = schema_url.path();
                println!("Loading schema from file: {}", path);
                Self::load_from_file(path)
            }
            _ => {
                Err(Error::SchemaNotFound {
                    path_or_url: schema_url.to_string(),
                    error: format!("Unsupported URL scheme: {}", schema_url.scheme()),
                })
            }
        }
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
