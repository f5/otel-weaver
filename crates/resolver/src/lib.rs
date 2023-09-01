use std::path::Path;

use schema::TelemetrySchema;

pub struct SchemaResolver {}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("telemetry schema error (error: {0:?})")]
    TelemetrySchemaError(schema::Error),

    #[error("parent schema error (error: {0:?})")]
    ParentSchemaError(schema::Error),
}

impl SchemaResolver {
    pub fn resolve_schema_file<P: AsRef<Path>>(schema_path: P) -> Result<TelemetrySchema, Error> {
        let schema = TelemetrySchema::load_from_file(schema_path)
            .map_err(|e| Error::TelemetrySchemaError(e))?;

        // Load the parent schema and merge it into the current schema.
        let parent_schema = if let Some(parent_schema_url) = schema.parent_schema_url.as_ref() {
            Some(TelemetrySchema::load_from_url(parent_schema_url)
                .map_err(|e| Error::ParentSchemaError(e))?)
        } else {
            None
        };

        // Load all the semantic convention catalogs.
        // Resolve the references to the semantic conventions.

        return Ok(schema);
    }
}

#[cfg(test)]
mod test {
    use crate::SchemaResolver;

    #[test]
    fn resolve_schema() {
        let schema = SchemaResolver::resolve_schema_file("../../data/app-telemetry-schema.yaml");
        assert!(schema.is_ok(), "{:#?}", schema.err().unwrap());
    }
}