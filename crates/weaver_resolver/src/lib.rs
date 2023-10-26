// SPDX-License-Identifier: Apache-2.0

//! This crate implements the process of reference resolution for telemetry schemas.

#![deny(missing_docs)]
#![deny(clippy::print_stdout)]
#![deny(clippy::print_stderr)]

use std::path::Path;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::time::Instant;

use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use regex::Regex;
use url::Url;
use walkdir::DirEntry;

use weaver_cache::Cache;
use weaver_logger::Logger;
use weaver_schema::{SemConvImport, TelemetrySchema};
use weaver_semconv::{ResolverConfig, SemConvCatalog, SemConvSpec};
use weaver_version::VersionChanges;

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
    TelemetrySchemaError(weaver_schema::Error),

    /// A parent schema error.
    #[error("Parent schema error (error: {0:?})")]
    ParentSchemaError(weaver_schema::Error),

    /// An invalid URL.
    #[error("Invalid URL `{url:?}`, error: {error:?})")]
    InvalidUrl {
        /// The invalid URL.
        url: String,
        /// The error that occurred.
        error: String,
    },

    /// A semantic convention error.
    #[error("Semantic convention error: {message}")]
    SemConvError {
        /// The error that occurred.
        message: String,
    },

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
        cache: &Cache,
        log: impl Logger + Clone + Sync,
    ) -> Result<TelemetrySchema, Error> {
        let mut schema = Self::load_schema_from_path(schema_path.clone(), log.clone())?;
        let sem_conv_catalog = Self::semantic_catalog_from_schema(&schema, cache, log.clone())?;
        let start = Instant::now();

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
            "Resolved schema '{}' ({:.2}s)",
            schema_path.as_ref().display(),
            start.elapsed().as_secs_f32()
        ));

        schema.semantic_conventions.clear();
        schema.set_semantic_convention_catalog(sem_conv_catalog);

        Ok(schema)
    }

    /// Loads a telemetry schema from the given path.
    pub fn load_schema_from_path<P: AsRef<Path> + Clone>(
        schema_path: P,
        log: impl Logger + Clone + Sync,
    ) -> Result<TelemetrySchema, Error> {
        let start = Instant::now();
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
            "Loaded schema '{}' ({:.2}s)",
            schema_path.as_ref().display(),
            start.elapsed().as_secs_f32()
        ));

        let parent_schema = Self::load_parent_schema(&schema, log.clone())?;
        schema.set_parent_schema(parent_schema);
        Ok(schema)
    }

    /// Loads a semantic convention catalog from the given schema path.
    pub fn semantic_catalog_from_schema(
        schema: &TelemetrySchema,
        cache: &Cache,
        log: impl Logger + Clone + Sync,
    ) -> Result<SemConvCatalog, Error> {
        let start = Instant::now();
        let semantic_conventions = schema.merged_semantic_conventions();
        let mut sem_conv_catalog =
            Self::create_semantic_convention_catalog(&semantic_conventions, cache, log.clone())?;
        let _ = sem_conv_catalog
            .resolve(ResolverConfig::default())
            .map_err(|e| Error::SemConvError {
                message: e.to_string(),
            })?;
        log.success(&format!(
            "Loaded {} semantic convention files containing the definition of {} attributes and {} metrics ({:.2}s)",
            sem_conv_catalog.asset_count(),
            sem_conv_catalog.attribute_count(),
            sem_conv_catalog.metric_count(),
            start.elapsed().as_secs_f32()
        ));

        Ok(sem_conv_catalog)
    }

    /// Loads the parent telemetry schema if it exists.
    fn load_parent_schema(
        schema: &TelemetrySchema,
        log: impl Logger,
    ) -> Result<Option<TelemetrySchema>, Error> {
        let start = Instant::now();
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

            log.success(&format!(
                "Loaded parent schema '{}' ({:.2}s)",
                parent_schema_url,
                start.elapsed().as_secs_f32()
            ));
            Some(parent_schema)
        } else {
            None
        };

        Ok(parent_schema)
    }

    /// Creates a semantic convention catalog from the given telemetry schema.
    fn create_semantic_convention_catalog(
        sem_convs: &[SemConvImport],
        cache: &Cache,
        log: impl Logger + Sync,
    ) -> Result<SemConvCatalog, Error> {
        // Load all the semantic convention catalogs.
        let mut sem_conv_catalog = SemConvCatalog::default();
        let total_file_count = sem_convs.len();
        let loaded_files_count = AtomicUsize::new(0);
        let error_count = AtomicUsize::new(0);

        let result: Vec<Result<(String, SemConvSpec), Error>> = sem_convs
            .par_iter()
            .flat_map(|sem_conv_import| {
                let results = Self::import_sem_conv_specs(sem_conv_import, cache);
                for result in results.iter() {
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
                }
                results
            })
            .collect();

        let mut errors = vec![];
        result.into_iter().for_each(|result| match result {
            Ok(sem_conv_spec) => {
                sem_conv_catalog.append_sem_conv_spec(sem_conv_spec);
            }
            Err(e) => {
                log.error(&e.to_string());
                errors.push(e);
            }
        });

        // ToDo LQ: Propagate the errors!

        Ok(sem_conv_catalog)
    }

    /// Imports the semantic convention specifications from the given import declaration.
    /// This function returns a vector of results because the import declaration can be a
    /// URL or a git URL (containing potentially multiple semantic convention specifications).
    fn import_sem_conv_specs(
        import_decl: &SemConvImport,
        cache: &Cache,
    ) -> Vec<Result<(String, SemConvSpec), Error>> {
        match import_decl {
            SemConvImport::Url { url } => {
                let spec = SemConvCatalog::load_sem_conv_spec_from_url(url).map_err(|e| {
                    Error::SemConvError {
                        message: e.to_string(),
                    }
                });
                vec![spec]
            }
            SemConvImport::GitUrl { git_url, path } => {
                fn is_hidden(entry: &DirEntry) -> bool {
                    entry
                        .file_name()
                        .to_str()
                        .map(|s| s.starts_with('.'))
                        .unwrap_or(false)
                }
                fn is_semantic_convention_file(entry: &DirEntry) -> bool {
                    let path = entry.path();
                    let extension = path.extension().unwrap_or_else(|| std::ffi::OsStr::new(""));
                    let file_name = path.file_name().unwrap_or_else(|| std::ffi::OsStr::new(""));
                    path.is_file()
                        && (extension == "yaml" || extension == "yml")
                        && file_name != "schema-next.yaml"
                }

                let mut result = vec![];
                let git_repo = cache.git_repo(git_url.clone(), path.clone()).map_err(|e| {
                    Error::SemConvError {
                        message: e.to_string(),
                    }
                });

                if let Ok(git_repo) = git_repo {
                    // Loads the semantic convention specifications from the git repo.
                    // All yaml files are recursively loaded from the given path.
                    for entry in walkdir::WalkDir::new(git_repo)
                        .into_iter()
                        .filter_entry(|e| !is_hidden(e))
                    {
                        match entry {
                            Ok(entry) => {
                                if is_semantic_convention_file(&entry) {
                                    let spec =
                                        SemConvCatalog::load_sem_conv_spec_from_file(entry.path())
                                            .map_err(|e| Error::SemConvError {
                                                message: e.to_string(),
                                            });
                                    result.push(spec);
                                }
                            }
                            Err(e) => result.push(Err(Error::SemConvError {
                                message: e.to_string(),
                            })),
                        }
                    }
                }

                result
            }
        }
    }
}

#[cfg(test)]
mod test {
    use weaver_cache::Cache;
    use weaver_logger::{ConsoleLogger, Logger};

    use crate::SchemaResolver;

    #[test]
    fn resolve_schema() {
        let log = ConsoleLogger::new(0);
        let mut cache = Cache::try_new().unwrap_or_else(|e| {
            log.error(&e.to_string());
            std::process::exit(1);
        });
        let schema = SchemaResolver::resolve_schema_file(
            "../../data/app-telemetry-schema.yaml",
            &mut cache,
            log,
        );
        assert!(schema.is_ok(), "{:#?}", schema.err().unwrap());
    }
}
