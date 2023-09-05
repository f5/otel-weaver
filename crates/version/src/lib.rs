// SPDX-License-Identifier: Apache-2.0

//! The specification of the changes to apply to the schema for different versions.

#![deny(missing_docs)]
#![deny(clippy::print_stdout)]

use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::logs_version::LogsVersion;
use crate::metrics_version::MetricsVersion;
use crate::resource_version::ResourceVersion;
use crate::spans_version::SpansVersion;

pub mod logs_change;
pub mod logs_version;
pub mod metrics_change;
pub mod metrics_version;
pub mod resource_change;
pub mod resource_version;
pub mod spans_change;
pub mod spans_version;

/// An error that can occur while loading or resolving version changes.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// The `versions` file was not found.
    #[error("Versions {path_or_url:?} not found\n{error:?}")]
    VersionsNotFound {
        /// The path or URL of the `versions` file.
        path_or_url: String,
        /// The error that occurred.
        error: String
    },

    /// The `versions` file is invalid.
    #[error("Invalid versions {path_or_url:?}\n{error:?}")]
    InvalidVersions {
        /// The path or URL of the `versions` file.
        path_or_url: String,
        /// The line number where the error occurred.
        line: Option<usize>,
        /// The column number where the error occurred.
        column: Option<usize>,
        /// The error that occurred.
        error: String
    },
}

/// List of versions with their changes.
#[derive(serde::Deserialize, Debug)]
struct Versions {
    versions: BTreeMap<semver::Version, VersionSpec>,

    #[serde(skip)]
    resource_old_to_new_attributes: HashMap<String, String>,
    #[serde(skip)]
    metrics_old_to_new_names: HashMap<String, String>,
    #[serde(skip)]
    logs_old_to_new_attributes: HashMap<String, String>,
    #[serde(skip)]
    spans_old_to_new_attributes: HashMap<String, String>,
}

/// An history of changes to apply to the schema for different versions.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct VersionSpec {
    /// The changes to apply to the metrics specification for a specific version.
    pub metrics: Option<MetricsVersion>,
    /// The changes to apply to the logs specification for a specific version.
    pub logs: Option<LogsVersion>,
    /// The changes to apply to the spans specification for a specific version.
    pub spans: Option<SpansVersion>,
    /// The changes to apply to the resource specification for a specific version.
    pub resources: Option<ResourceVersion>,
}

impl Versions {
    /// Loads a `versions` file and returns an instance of `Versions` if successful
    /// or an error if the file could not be loaded or deserialized.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Versions, Error> {
        let path_buf = path.as_ref().to_path_buf();

        // Load and deserialize the telemetry schema
        let versions_file = File::open(path).map_err(|e| Error::VersionsNotFound {
            path_or_url: path_buf.as_path().display().to_string(),
            error: e.to_string(),
        })?;
        let versions: Versions = serde_yaml::from_reader(BufReader::new(versions_file))
            .map_err(|e| Error::InvalidVersions {
                path_or_url: path_buf.as_path().display().to_string(),
                line: e.location().map(|loc| loc.line()),
                column: e.location().map(|loc| loc.column()),
                error: e.to_string(),
            })?;
        Ok(versions)
    }

    /// Returns a vector of tuples containing the versions and their corresponding changes
    /// in ascending order.
    pub fn versions_asc(&self) -> Vec<(&semver::Version, &VersionSpec)> {
        self.versions.iter().collect()
    }

    /// Returns a vector of tuples containing the versions and their corresponding changes
    /// in descending order.
    pub fn versions_desc(&self) -> Vec<(&semver::Version, &VersionSpec)> {
        self.versions.iter().rev().collect()
    }

    /// Resolves the transformations to get the most up-to-date attributes and
    /// metric names.
    pub fn resolve(&mut self) {
        // Builds a map of old to new attribute names for the attributes that have been renamed
        // in the different versions of the resources.
        let mut resource_old_to_new_attributes: HashMap<String, String> = HashMap::new();
        for (_, spec) in self.versions_desc() {
            if let Some(resources) = spec.resources.as_ref() {
                resources.changes.iter().flat_map(|change| change.rename_attributes.attribute_map.iter())
                    .for_each(|(old_name, new_name)| {
                        if !resource_old_to_new_attributes.contains_key(old_name) {
                            resource_old_to_new_attributes.insert(old_name.clone(), new_name.clone());
                        }
                });
            }
        }
        self.resource_old_to_new_attributes = resource_old_to_new_attributes;

        // Builds a map of old to new metric names that have been renamed
        // in the different versions.
        let mut metrics_old_to_new_names: HashMap<String, String> = HashMap::new();
        for (_, spec) in self.versions_desc() {
            if let Some(metrics) = spec.metrics.as_ref() {
                metrics.changes.iter().flat_map(|change| change.rename_metrics.iter())
                    .for_each(|(old_name, new_name)| {
                        if !metrics_old_to_new_names.contains_key(old_name) {
                            metrics_old_to_new_names.insert(old_name.clone(), new_name.clone());
                        }
                });
            }
        }
        self.metrics_old_to_new_names = metrics_old_to_new_names;

        // Builds a map of old to new attribute names for the attributes that have been renamed
        // in the different versions of the logs.
        let mut logs_old_to_new_attributes: HashMap<String, String> = HashMap::new();
        for (_, spec) in self.versions_desc() {
            if let Some(logs) = spec.logs.as_ref() {
                logs.changes.iter().flat_map(|change| change.rename_attributes.attribute_map.iter())
                    .for_each(|(old_name, new_name)| {
                        if !logs_old_to_new_attributes.contains_key(old_name) {
                            logs_old_to_new_attributes.insert(old_name.clone(), new_name.clone());
                        }
                });
            }
        }
        self.logs_old_to_new_attributes = logs_old_to_new_attributes;

        // Builds a map of old to new attribute names for the attributes that have been renamed
        // in the different versions of the spans.
        let mut spans_old_to_new_attributes: HashMap<String, String> = HashMap::new();
        for (_, spec) in self.versions_desc() {
            if let Some(spans) = spec.spans.as_ref() {
                spans.changes.iter().flat_map(|change| change.rename_attributes.attribute_map.iter())
                    .for_each(|(old_name, new_name)| {
                        if !spans_old_to_new_attributes.contains_key(old_name) {
                            spans_old_to_new_attributes.insert(old_name.clone(), new_name.clone());
                        }
                });
            }
        }
        self.spans_old_to_new_attributes = spans_old_to_new_attributes;
    }

    /// Returns the new name of the given resource attribute or the given name if the attribute
    /// has not been renamed.
    pub fn get_resource_attribute_name(&self, name: &str) -> String {
        if let Some(new_name) = self.resource_old_to_new_attributes.get(name) {
            new_name.clone()
        } else {
            name.to_string()
        }
    }

    /// Returns the new name of the given metric or the given name if the metric
    /// has not been renamed.
    pub fn get_metric_name(&self, name: &str) -> String {
        if let Some(new_name) = self.metrics_old_to_new_names.get(name) {
            new_name.clone()
        } else {
            name.to_string()
        }
    }

    /// Returns the new name of the given log attribute or the given name if the attribute
    /// has not been renamed.
    pub fn get_log_attribute_name(&self, name: &str) -> String {
        if let Some(new_name) = self.logs_old_to_new_attributes.get(name) {
            new_name.clone()
        } else {
            name.to_string()
        }
    }

    /// Returns the new name of the given span attribute or the given name if the attribute
    /// has not been renamed.
    pub fn get_span_attribute_name(&self, name: &str) -> String {
        if let Some(new_name) = self.spans_old_to_new_attributes.get(name) {
            new_name.clone()
        } else {
            name.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Versions};

    #[test]
    fn test_ordering() {
        let versions: Versions = Versions::load_from_file("data/versions.yaml").unwrap();
        let mut version = None;

        for (v, _) in versions.versions_asc() {
            if version.is_some() {
                assert!(v > version.unwrap());
            }
            version = Some(v);
        }

        let mut version = None;

        for (v, _) in versions.versions_desc() {
            if version.is_some() {
                assert!(v < version.unwrap());
            }
            version = Some(v);
        }
    }

    #[test]
    fn test_resolve() {
        let mut versions: Versions = Versions::load_from_file("data/versions.yaml").unwrap();

        versions.resolve();

        // Test renaming of resource attributes
        assert_eq!("user_agent.original", versions.get_resource_attribute_name("browser.user_agent"));

        // Test renaming of metric names
        assert_eq!("process.runtime.jvm.cpu.recent_utilization", versions.get_metric_name("process.runtime.jvm.cpu.utilization"));

        // Test renaming of span attributes
        assert_eq!("user_agent.original", versions.get_span_attribute_name("http.user_agent"));
        assert_eq!("http.request.method", versions.get_span_attribute_name("http.method"));
        assert_eq!("url.full", versions.get_span_attribute_name("http.url"));
        assert_eq!("net.protocol.name", versions.get_span_attribute_name("net.app.protocol.name"));
        assert_eq!("cloud.resource_id", versions.get_span_attribute_name("faas.id"));
        assert_eq!("db.name", versions.get_span_attribute_name("db.hbase.namespace"));
        assert_eq!("db.name", versions.get_span_attribute_name("db.cassandra.keyspace"));
    }
}