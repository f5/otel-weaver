// SPDX-License-Identifier: Apache-2.0

//! Resolve events

use crate::attribute::resolve_attributes;
use crate::Error;
use schema::schema_spec::SchemaSpec;
use semconv::SemConvCatalog;
use version::VersionChanges;

/// Resolves resource events and their attributes.
pub fn resolve_events(
    schema: &mut SchemaSpec,
    sem_conv_catalog: &mut SemConvCatalog,
    version_changes: &VersionChanges,
) -> Result<(), Error> {
    if let Some(events) = schema.resource_events.as_mut() {
        events.attributes = resolve_attributes(
            events.attributes.as_ref(),
            sem_conv_catalog,
            version_changes.log_attribute_changes(),
        )?;
        for event in events.events.iter_mut() {
            event.attributes = resolve_attributes(
                event.attributes.as_ref(),
                sem_conv_catalog,
                version_changes.log_attribute_changes(),
            )?;
        }
    }
    Ok(())
}
