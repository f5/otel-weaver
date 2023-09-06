// SPDX-License-Identifier: Apache-2.0

//! The specification of the changes to apply to the schema for different versions.

#![deny(missing_docs)]
#![deny(clippy::print_stdout)]

use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::logs_change::LogsChange;
use crate::logs_version::LogsVersion;
use crate::metrics_change::MetricsChange;
use crate::metrics_version::MetricsVersion;
use crate::resource_change::ResourceChange;
use crate::resource_version::ResourceVersion;
use crate::spans_change::SpansChange;
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
        error: String,
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
        error: String,
    },
}

/// List of versions with their changes.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(transparent)]
pub struct Versions {
    versions: BTreeMap<semver::Version, VersionSpec>,
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

/// The changes to apply to rename attributes and metrics for
/// a specific version.
pub struct VersionChanges {
    version: semver::Version,
    resource_old_to_new_attributes: HashMap<String, String>,
    metrics_old_to_new_names: HashMap<String, String>,
    logs_old_to_new_attributes: HashMap<String, String>,
    spans_old_to_new_attributes: HashMap<String, String>,
}

impl Versions {
    /// Loads a `versions` file and returns an instance of `Versions` if successful
    /// or an error if the file could not be loaded or deserialized.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Versions, Error> {
        /// Versions has a transparent serde representation so we need to define a top-level
        /// struct to deserialize the `versions` file.
        #[derive(Serialize, Deserialize, Debug)]
        struct TopLevel {
            versions: Versions,
        }

        let path_buf = path.as_ref().to_path_buf();

        // Load and deserialize the telemetry schema
        let versions_file = File::open(path).map_err(|e| Error::VersionsNotFound {
            path_or_url: path_buf.as_path().display().to_string(),
            error: e.to_string(),
        })?;
        let top_level: TopLevel = serde_yaml::from_reader(BufReader::new(versions_file))
            .map_err(|e| Error::InvalidVersions {
                path_or_url: path_buf.as_path().display().to_string(),
                line: e.location().map(|loc| loc.line()),
                column: e.location().map(|loc| loc.column()),
                error: e.to_string(),
            })?;
        Ok(top_level.versions)
    }

    /// Returns the most recent version or None if there are no versions.
    pub fn latest_version(&self) -> Option<&semver::Version> {
        self.versions.keys().last()
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

    /// Returns a vector of tuples containing the versions and their corresponding changes
    /// in ascending order from the given version.
    pub fn versions_asc_from(&self, version: &semver::Version) -> Vec<(&semver::Version, &VersionSpec)> {
        self.versions.range(version.clone()..).collect()
    }

    /// Returns a vector of tuples containing the versions and their corresponding changes
    /// in descending order from the given version.
    pub fn versions_desc_from(&self, version: &semver::Version) -> Vec<(&semver::Version, &VersionSpec)> {
        self.versions.range(..=version.clone()).rev().collect()
    }

    /// Returns the changes to apply for the given version including the changes
    /// of the previous versions.
    /// The current supported changes are:
    /// - Renaming of attributes (for resources, logs and spans)
    /// - Renaming of metrics
    pub fn version_changes_for(&self, version: &semver::Version) -> VersionChanges {
        let mut resource_old_to_new_attributes: HashMap<String, String> = HashMap::new();
        let mut metrics_old_to_new_names: HashMap<String, String> = HashMap::new();
        let mut logs_old_to_new_attributes: HashMap<String, String> = HashMap::new();
        let mut spans_old_to_new_attributes: HashMap<String, String> = HashMap::new();

        for (_, spec) in self.versions_desc_from(version) {
            // Builds a map of old to new attribute names for the attributes that have been renamed
            // in the different versions of the resources.
            if let Some(resources) = spec.resources.as_ref() {
                resources.changes.iter().flat_map(|change| change.rename_attributes.attribute_map.iter())
                    .for_each(|(old_name, new_name)| {
                        if !resource_old_to_new_attributes.contains_key(old_name) {
                            resource_old_to_new_attributes.insert(old_name.clone(), new_name.clone());
                        }
                    });
            }

            // Builds a map of old to new metric names that have been renamed
            // in the different versions.
            if let Some(metrics) = spec.metrics.as_ref() {
                metrics.changes.iter().flat_map(|change| change.rename_metrics.iter())
                    .for_each(|(old_name, new_name)| {
                        if !metrics_old_to_new_names.contains_key(old_name) {
                            metrics_old_to_new_names.insert(old_name.clone(), new_name.clone());
                        }
                    });
            }

            // Builds a map of old to new attribute names for the attributes that have been renamed
            // in the different versions of the logs.
            if let Some(logs) = spec.logs.as_ref() {
                logs.changes.iter().flat_map(|change| change.rename_attributes.attribute_map.iter())
                    .for_each(|(old_name, new_name)| {
                        if !logs_old_to_new_attributes.contains_key(old_name) {
                            logs_old_to_new_attributes.insert(old_name.clone(), new_name.clone());
                        }
                    });
            }

            // Builds a map of old to new attribute names for the attributes that have been renamed
            // in the different versions of the spans.
            if let Some(spans) = spec.spans.as_ref() {
                spans.changes.iter().flat_map(|change| change.rename_attributes.attribute_map.iter())
                    .for_each(|(old_name, new_name)| {
                        if !spans_old_to_new_attributes.contains_key(old_name) {
                            spans_old_to_new_attributes.insert(old_name.clone(), new_name.clone());
                        }
                    });
            }
        }

        return VersionChanges {
            version: version.clone(),
            resource_old_to_new_attributes,
            metrics_old_to_new_names,
            logs_old_to_new_attributes,
            spans_old_to_new_attributes,
        };
    }

    /// Update the current `Versions` to include the transformations of the parent `Versions`.
    /// Transformations of the current `Versions` take precedence over the parent `Versions`.
    pub fn extend(&mut self, parent_versions: Versions) {
        for (version, spec) in parent_versions.versions.into_iter() {
            match self.versions.get_mut(&version) {
                Some(current_spec) => {
                    current_spec.extend(spec);
                }
                None => {
                    self.versions.insert(version.clone(), spec);
                }
            }
        }
    }
}

impl VersionSpec {
    /// Update the current `VersionSpec` to include the transformations of the parent `VersionSpec`.
    /// Transformations of the current `VersionSpec` take precedence over the parent `VersionSpec`.
    pub fn extend(&mut self, parent_spec: VersionSpec) {
        // Process resources
        if let Some(resources) = parent_spec.resources {
            let mut resource_change = ResourceChange::default();
            for change in resources.changes {
                'next_parent_renaming: for (old, new) in change.rename_attributes.attribute_map {
                    for local_change in self.resources.get_or_insert_with(|| ResourceVersion::default()).changes.iter() {
                        if local_change.rename_attributes.attribute_map.contains_key(&old) {
                            // renaming already present in local changes, skip it
                            continue 'next_parent_renaming;
                        }
                    }
                    // renaming not found in local changes, add it
                    resource_change.rename_attributes.attribute_map.insert(old, new);
                }
            }
            if resource_change.rename_attributes.attribute_map.len() > 0 {
                self.resources.get_or_insert_with(|| ResourceVersion::default()).changes.push(resource_change);
            }
        }

        // Process metrics
        if let Some(metrics) = parent_spec.metrics {
            let mut metrics_change = MetricsChange::default();
            for change in metrics.changes {
                'next_parent_renaming: for (old, new) in change.rename_metrics {
                    for local_change in self.metrics.get_or_insert_with(|| MetricsVersion::default()).changes.iter() {
                        if local_change.rename_metrics.contains_key(&old) {
                            // renaming already present in local changes, skip it
                            continue 'next_parent_renaming;
                        }
                    }
                    // renaming not found in local changes, add it
                    metrics_change.rename_metrics.insert(old, new);
                }
            }
            if metrics_change.rename_metrics.len() > 0 {
                self.metrics.get_or_insert_with(|| MetricsVersion::default()).changes.push(metrics_change);
            }
        }

        // Process logs
        if let Some(logs) = parent_spec.logs {
            let mut logs_change = LogsChange::default();
            for change in logs.changes {
                'next_parent_renaming: for (old, new) in change.rename_attributes.attribute_map {
                    for local_change in self.logs.get_or_insert_with(|| LogsVersion::default()).changes.iter() {
                        if local_change.rename_attributes.attribute_map.contains_key(&old) {
                            // renaming already present in local changes, skip it
                            continue 'next_parent_renaming;
                        }
                    }
                    // renaming not found in local changes, add it
                    logs_change.rename_attributes.attribute_map.insert(old, new);
                }
            }
            if logs_change.rename_attributes.attribute_map.len() > 0 {
                self.logs.get_or_insert_with(|| LogsVersion::default()).changes.push(logs_change);
            }
        }

        // Process spans
        if let Some(spans) = parent_spec.spans {
            let mut spans_change = SpansChange::default();
            for change in spans.changes {
                'next_parent_renaming: for (old, new) in change.rename_attributes.attribute_map {
                    for local_change in self.spans.get_or_insert_with(|| SpansVersion::default()).changes.iter() {
                        if local_change.rename_attributes.attribute_map.contains_key(&old) {
                            // renaming already present in local changes, skip it
                            continue 'next_parent_renaming;
                        }
                    }
                    // renaming not found in local changes, add it
                    spans_change.rename_attributes.attribute_map.insert(old, new);
                }
            }
            if spans_change.rename_attributes.attribute_map.len() > 0 {
                self.spans.get_or_insert_with(|| SpansVersion::default()).changes.push(spans_change);
            }
        }
    }
}

impl VersionChanges {
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
    use crate::Versions;

    #[test]
    fn test_ordering() {
        let versions: Versions = Versions::load_from_file("data/parent_versions.yaml").unwrap();
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
    fn test_version_changes_for() {
        let versions: Versions = Versions::load_from_file("data/parent_versions.yaml").unwrap();
        let changes = versions.version_changes_for(versions.latest_version().unwrap());

        // Test renaming of resource attributes
        assert_eq!("user_agent.original", changes.get_resource_attribute_name("browser.user_agent"));

        // Test renaming of metric names
        assert_eq!("process.runtime.jvm.cpu.recent_utilization", changes.get_metric_name("process.runtime.jvm.cpu.utilization"));

        // Test renaming of span attributes
        assert_eq!("user_agent.original", changes.get_span_attribute_name("http.user_agent"));
        assert_eq!("http.request.method", changes.get_span_attribute_name("http.method"));
        assert_eq!("url.full", changes.get_span_attribute_name("http.url"));
        assert_eq!("net.protocol.name", changes.get_span_attribute_name("net.app.protocol.name"));
        assert_eq!("cloud.resource_id", changes.get_span_attribute_name("faas.id"));
        assert_eq!("db.name", changes.get_span_attribute_name("db.hbase.namespace"));
        assert_eq!("db.name", changes.get_span_attribute_name("db.cassandra.keyspace"));
    }

    #[test]
    fn test_override() {
        let parent_versions = Versions::load_from_file("data/parent_versions.yaml").unwrap();
        let mut app_versions = Versions::load_from_file("data/app_versions.yaml").unwrap();

        // Update `app_version` to extend `parent_versions`
        app_versions.extend(parent_versions);

        dbg!(&app_versions);

        // Transformations defined in `app_versions.yaml` masking or
        // complementing `parent_versions.yaml`
        let v1_22 = app_versions.versions
            .get(&semver::Version::parse("1.22.0").unwrap()).unwrap();
        let observed_value = v1_22.spans.as_ref().unwrap()
            .changes[0]
            .rename_attributes.attribute_map.get("messaging.kafka.client_id");
        assert_eq!(observed_value, Some(&"messaging.client.id".to_string()));

        let v1_8 = app_versions.versions
            .get(&semver::Version::parse("1.8.0").unwrap()).unwrap();
        let observed_value = v1_8.spans.as_ref().unwrap()
            .changes[0]
            .rename_attributes.attribute_map.get("db.cassandra.keyspace");
        assert_eq!(observed_value, Some(&"database.name".to_string()));
        let observed_value = v1_8.resources.as_ref().unwrap()
            .changes[0]
            .rename_attributes.attribute_map.get("db.cassandra.db");
        assert_eq!(observed_value, Some(&"database.name".to_string()));

        let v1_7_1 = app_versions.versions
            .get(&semver::Version::parse("1.7.1").unwrap()).unwrap();
        let observed_value = v1_7_1.spans.as_ref().unwrap()
            .changes[0]
            .rename_attributes.attribute_map.get("db.cassandra.table");
        assert_eq!(observed_value, Some(&"database.table".to_string()));

        // Transformations inherited from `parent_versions.yaml` and
        // initially not present in `app_versions.yaml`
        let v1_21 = app_versions.versions
            .get(&semver::Version::parse("1.21.0").unwrap()).unwrap();
        let observed_value = v1_21.metrics.as_ref().unwrap()
            .changes[0]
            .rename_metrics.get("process.runtime.jvm.cpu.utilization");
        assert_eq!(observed_value, Some(&"process.runtime.jvm.cpu.recent_utilization".to_string()));
        let observed_value = v1_21.spans.as_ref().unwrap()
            .changes[0]
            .rename_attributes.attribute_map.get("messaging.kafka.client_id");
        assert_eq!(observed_value, Some(&"messaging.client_id".to_string()));

        // messaging.consumer_id: messaging.consumer.id
    }
}