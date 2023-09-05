// SPDX-License-Identifier: Apache-2.0

//! A resolver that can be used to resolve telemetry schemas.

#![deny(missing_docs)]
#![deny(clippy::print_stdout)]

use logger::Logger;
use std::path::Path;

use schema::TelemetrySchema;
use semconv::attribute::Attribute;

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

    /// A semantic convention error.
    #[error("Semantic convention error (error: {0:?})")]
    SemConvError(semconv::Error),

    /// Failed to resolve an attribute.
    #[error("Failed to resolve attribute '{r#ref}'")]
    FailToResolveAttribute {
        /// The reference to the attribute.
        r#ref: String,
    }
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

        let parent_schema = Self::load_parent_schema(&schema, log)?;
        let sem_conv_catalog = Self::create_semantic_convention_catalog(&schema, log)?;

        // Resolve the references to the semantic conventions.
        log.loading("Solving semantic convention references");
        if let Some(schema) = schema.schema.as_mut() {
            // Resolve common attributes
            if let Some(metrics) = schema.resource_metrics.as_mut() {
                Self::resolve_attributes(metrics.attributes.as_mut(), &sem_conv_catalog)?;
            }
            if let Some(logs) = schema.resource_logs.as_mut() {
                Self::resolve_attributes(logs.attributes.as_mut(), &sem_conv_catalog)?;
            }
            if let Some(spans) = schema.resource_spans.as_mut() {
                Self::resolve_attributes(spans.attributes.as_mut(), &sem_conv_catalog)?;
            }
            // Merge common attributes with the attributes of the corresponding resource_metrics,
            // resource_logs and resource_spans.
            if let Some(logs) = schema.resource_logs.as_mut() {
                for log in logs.logs.iter_mut() {
                    Self::resolve_attributes(log.attributes.as_mut(), &sem_conv_catalog)?;
                }
            }
        }
        log.success(&format!(
            "Resolved schema '{}'",
            schema_path.as_ref().display()
        ));

        Ok(schema)
    }

    /// Loads the parent telemetry schema if it exists.
    fn load_parent_schema(schema: &TelemetrySchema, log: &mut Logger) -> Result<Option<TelemetrySchema>, Error> {
        // Load the parent schema and merge it into the current schema.
        let parent_schema = if let Some(parent_schema_url) = schema.parent_schema_url.as_ref() {
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

        Ok(parent_schema)
    }

    /// Creates a semantic convention catalog from the given telemetry schema.
    fn create_semantic_convention_catalog(schema: &TelemetrySchema, log: &mut Logger) -> Result<semconv::SemConvCatalog, Error> {
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

        Ok(sem_conv_catalog)
    }

    fn resolve_attributes(attributes: &mut Vec<Attribute>, sem_conv_catalog: &semconv::SemConvCatalog) -> Result<(), Error> {
        for attribute in attributes.iter_mut() {
            if let Some(r#ref) = attribute.r#ref.as_ref() {
                if let Some(resolved_attribute) = sem_conv_catalog.get_attribute(r#ref) {
                    *attribute = resolved_attribute;
                } else {
                    return Err(Error::FailToResolveAttribute {
                        r#ref: r#ref.clone(),
                    });
                }
            }
        }
        Ok(())
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
