// SPDX-License-Identifier: Apache-2.0

//! This crate implements the process of reference resolution for telemetry schemas.

#![deny(missing_docs)]
#![deny(clippy::print_stdout)]
#![deny(clippy::print_stderr)]

use std::path::Path;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;

use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use regex::Regex;
use url::Url;

use logger::Logger;
use schema::SemConvImport;
pub use schema::TelemetrySchema;
use semconv::{ResolverConfig, SemConvCatalog, SemConvSpec};
use version::VersionChanges;

use crate::resource::resolve_resource;
use crate::resource_events::resolve_events;
use crate::resource_metrics::resolve_metrics;
use crate::resource_spans::resolve_spans;

mod attribute;
mod resource;
mod resource_events;
mod resource_metrics;
mod resource_spans;

/// A resolver that can be used to resolve telemetry schemas.
/// All references to semantic conventions will be resolved.
pub struct SchemaResolver {}

/// An error that can occur while resolving a telemetry schema.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// A telemetry schema error.
    #[error("Telemetry schema error (error: {0:?})")]
    TelemetrySchemaError(schema::Error),

    /// A parent schema error.
    #[error("Parent schema error (error: {0:?})")]
    ParentSchemaError(schema::Error),

    /// An invalid URL.
    #[error("Invalid URL `{url:?}`, error: {error:?})")]
    InvalidUrl {
        /// The invalid URL.
        url: String,
        /// The error that occurred.
        error: String,
    },

    /// A semantic convention error.
    #[error("Semantic convention error (error: {0:?})")]
    SemConvError(semconv::Error),

    /// Failed to resolve an attribute.
    #[error("Failed to resolve the attribute '{id}'")]
    FailToResolveAttribute {
        /// The id of the attribute.
        id: String,
        /// The error that occurred.
        error: String,
    },

    /// Failed to resolve a metric.
    #[error("Failed to resolve the metric '{r#ref}'")]
    FailToResolveMetric {
        /// The reference to the metric.
        r#ref: String,
    },

    /// Metric attributes are incompatible within the metric group.
    #[error("Metric attributes are incompatible within the metric group '{metric_group_ref}' for metric '{metric_ref}' (error: {error})")]
    IncompatibleMetricAttributes {
        /// The metric group reference.
        metric_group_ref: String,
        /// The reference to the metric.
        metric_ref: String,
        /// The error that occurred.
        error: String,
    },
}

impl SchemaResolver {
    /// Loads a telemetry schema file and returns the resolved schema.
    pub fn resolve_schema_file<P: AsRef<Path> + Clone>(
        schema_path: P,
        log: &Logger,
    ) -> Result<TelemetrySchema, Error> {
        log.loading(&format!(
            "Loading schema '{}'",
            schema_path.as_ref().display()
        ));
        let mut schema = TelemetrySchema::load_from_file(schema_path.clone()).map_err(|e| {
            log.error(&format!(
                "Failed to load schema '{}'",
                schema_path.as_ref().display()
            ));
            Error::TelemetrySchemaError(e)
        })?;
        log.success(&format!(
            "Loaded schema '{}'",
            schema_path.as_ref().display()
        ));

        let parent_schema = Self::load_parent_schema(&schema, log.clone())?;
        schema.set_parent_schema(parent_schema);
        let semantic_conventions = schema.merged_semantic_conventions();
        let mut sem_conv_catalog =
            Self::create_semantic_convention_catalog(&semantic_conventions, log.clone())?;
        let _ = sem_conv_catalog
            .resolve(ResolverConfig::default())
            .map_err(Error::SemConvError)?;
        log.success(&format!(
            "Loaded {} semantic convention files ({} attributes, {} metrics)",
            semantic_conventions.len(),
            sem_conv_catalog.attribute_count(),
            sem_conv_catalog.metric_count()
        ));

        // Merges the versions of the parent schema into the current schema.
        schema.merge_versions();

        // Generates version changes
        let version_changes = schema
            .versions
            .as_ref()
            .map(|versions| {
                if let Some(latest_version) = versions.latest_version() {
                    versions.version_changes_for(latest_version)
                } else {
                    VersionChanges::default()
                }
            })
            .unwrap_or_default();

        // Resolve the references to the semantic conventions.
        log.loading("Solving semantic convention references");
        if let Some(schema) = schema.schema.as_mut() {
            resolve_resource(schema, &sem_conv_catalog, &version_changes)?;
            resolve_metrics(schema, &sem_conv_catalog, &version_changes)?;
            resolve_events(schema, &sem_conv_catalog, &version_changes)?;
            resolve_spans(schema, &sem_conv_catalog, version_changes)?;
        }
        log.success(&format!(
            "Resolved schema '{}'",
            schema_path.as_ref().display()
        ));

        schema.semantic_conventions.clear();

        Ok(schema)
    }

    /// Loads the parent telemetry schema if it exists.
    fn load_parent_schema(
        schema: &TelemetrySchema,
        log: Logger,
    ) -> Result<Option<TelemetrySchema>, Error> {
        // Load the parent schema and merge it into the current schema.
        let parent_schema = if let Some(parent_schema_url) = schema.parent_schema_url.as_ref() {
            log.loading(&format!("Loading parent schema '{}'", parent_schema_url));
            let url_pattern = Regex::new(r"^(https|http|file):.*")
                .expect("invalid regex, please report this bug");
            let parent_schema = if url_pattern.is_match(parent_schema_url) {
                let url = Url::parse(parent_schema_url).map_err(|e| {
                    log.error(&format!(
                        "Failed to parset parent schema url '{}'",
                        parent_schema_url
                    ));
                    Error::InvalidUrl {
                        url: parent_schema_url.clone(),
                        error: e.to_string(),
                    }
                })?;
                TelemetrySchema::load_from_url(&url).map_err(|e| {
                    log.error(&format!(
                        "Failed to load parent schema '{}'",
                        parent_schema_url
                    ));
                    Error::ParentSchemaError(e)
                })?
            } else {
                TelemetrySchema::load_from_file(parent_schema_url).map_err(|e| {
                    log.error(&format!(
                        "Failed to load parent schema '{}'",
                        parent_schema_url
                    ));
                    Error::ParentSchemaError(e)
                })?
            };

            log.success(&format!("Loaded schema '{}' (parent)", parent_schema_url));
            Some(parent_schema)
        } else {
            None
        };

        Ok(parent_schema)
    }

    /// Creates a semantic convention catalog from the given telemetry schema.
    fn create_semantic_convention_catalog(
        sem_convs: &[SemConvImport],
        log: Logger,
    ) -> Result<SemConvCatalog, Error> {
        // Load all the semantic convention catalogs.
        let mut sem_conv_catalog = SemConvCatalog::default();
        let total_file_count = sem_convs.len();
        let loaded_files_count = AtomicUsize::new(0);
        let error_count = AtomicUsize::new(0);

        let result: Vec<Result<(String, SemConvSpec), semconv::Error>> = sem_convs
            .par_iter()
            .map(|sem_conv_import| {
                let result = SemConvCatalog::load_sem_conv_spec_from_url(&sem_conv_import.url);
                if result.is_err() {
                    error_count.fetch_add(1, Relaxed);
                }
                loaded_files_count.fetch_add(1, Relaxed);
                if error_count.load(Relaxed) == 0 {
                    log.loading(&format!(
                        "Loaded {}/{} semantic convention files (no error detected)",
                        loaded_files_count.load(Relaxed),
                        total_file_count
                    ));
                } else {
                    log.loading(&format!(
                        "Loaded {}/{} semantic convention files ({} error(s) detected)",
                        loaded_files_count.load(Relaxed),
                        total_file_count,
                        error_count.load(Relaxed)
                    ));
                }
                result
            })
            .collect();

        let mut errors = vec![];
        result.into_iter().for_each(|result| match result {
            Ok(sem_conv_spec) => {
                sem_conv_catalog.append_sem_conv_spec(sem_conv_spec);
            }
            Err(e) => {
                log.error(&e.to_string());
                errors.push(Error::SemConvError(e));
            }
        });

        // ToDo do something with the errors

        Ok(sem_conv_catalog)
    }
}

#[cfg(test)]
mod test {
    use logger::Logger;

    use crate::SchemaResolver;

    #[test]
    fn resolve_schema() {
        let mut log = Logger::new(0);
        let schema =
            SchemaResolver::resolve_schema_file("../../data/app-telemetry-schema.yaml", &mut log);
        assert!(schema.is_ok(), "{:#?}", schema.err().unwrap());
    }
}
