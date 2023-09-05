// SPDX-License-Identifier: Apache-2.0

//! A resolver that can be used to resolve telemetry schemas.

#![deny(missing_docs)]
#![deny(clippy::print_stdout)]

use logger::Logger;
use std::path::Path;

use schema::TelemetrySchema;

/// A resolver that can be used to resolve telemetry schemas.
/// All references to semantic conventions will be resolved.
pub struct SchemaResolver {}

/// An error that can occur while resolving a telemetry schema.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// A telemetry schema error.
    #[error("telemetry schema error (error: {0:?})")]
    TelemetrySchemaError(schema::Error),

    /// A parent schema error.
    #[error("parent schema error (error: {0:?})")]
    ParentSchemaError(schema::Error),

    /// A semantic convention error.
    #[error("semantic convention error (error: {0:?})")]
    SemConvError(semconv::Error),
}

impl SchemaResolver {
    /// Loads a telemetry schema file and returns the resolved schema.
    pub fn resolve_schema_file<P: AsRef<Path> + Clone>(
        schema_path: P,
        log: &mut Logger,
    ) -> Result<TelemetrySchema, Error> {
        log.loading(&format!(
            "Loading schema '{}'",
            schema_path.as_ref().display()
        ));
        let schema = TelemetrySchema::load_from_file(schema_path.clone()).map_err(|e| {
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

        // Load the parent schema and merge it into the current schema.
        let _parent_schema = if let Some(parent_schema_url) = schema.parent_schema_url.as_ref() {
            log.loading(&format!("Loading parent schema '{}'", parent_schema_url));
            let parent_schema = TelemetrySchema::load_from_url(parent_schema_url).map_err(|e| {
                log.error(&format!(
                    "Failed to load parent schema '{}'",
                    parent_schema_url
                ));
                Error::ParentSchemaError(e)
            })?;
            log.success(&format!("Loaded parent schema '{}'", parent_schema_url));
            Some(parent_schema)
        } else {
            None
        };

        // Load all the semantic convention catalogs.
        let mut sem_conv_catalog = semconv::SemConvCatalog::default();
        log.loading(&format!(
            "Loading {} semantic convention catalogs",
            schema.semantic_conventions.len()
        ));
        for sem_conv_import in schema.semantic_conventions.iter() {
            sem_conv_catalog.load_from_url(&sem_conv_import.url).map_err(|e| {
                    log.error(&e.to_string());
                    Error::SemConvError(e)
                })?;
        }
        log.success(&format!(
            "Loaded {} semantic convention catalogs",
            schema.semantic_conventions.len()
        ));

        // Resolve the references to the semantic conventions.
        log.loading("Solving semantic convention references");
        log.success(&format!(
            "Resolved schema '{}'",
            schema_path.as_ref().display()
        ));

        Ok(schema)
    }
}

#[cfg(test)]
mod test {
    use crate::SchemaResolver;
    use logger::Logger;

    #[test]
    fn resolve_schema() {
        let mut log = Logger::new();
        let schema =
            SchemaResolver::resolve_schema_file("../../data/app-telemetry-schema.yaml", &mut log);
        assert!(schema.is_ok(), "{:#?}", schema.err().unwrap());
    }
}
