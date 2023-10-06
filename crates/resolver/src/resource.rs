// SPDX-License-Identifier: Apache-2.0

//! Resolve resource

use crate::attribute::resolve_attributes;
use crate::Error;
use schema::schema_spec::SchemaSpec;
use semconv::SemConvCatalog;
use version::VersionChanges;

/// Resolves resource attributes.
pub fn resolve_resource(
    schema: &mut SchemaSpec,
    sem_conv_catalog: &mut SemConvCatalog,
    version_changes: &VersionChanges,
) -> Result<(), Error> {
    // Resolve resource attributes
    if let Some(res) = schema.resource.as_mut() {
        res.attributes = resolve_attributes(
            res.attributes.as_ref(),
            &sem_conv_catalog,
            version_changes.log_attribute_changes(),
        )?;
    }
    Ok(())
}
