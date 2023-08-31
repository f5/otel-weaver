use std::path::Path;

use schema::TelemetrySchema;

pub struct SchemaResolver {}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    SchemaError(schema::Error),
}

impl SchemaResolver {
    pub fn resolve<P: AsRef<Path>>(schema_path: P) -> Result<TelemetrySchema, Error> {
        let schema = TelemetrySchema::load_from_file(schema_path)
            .map_err(|e| Error::SchemaError(e))?;
        return Ok(schema);
    }
}