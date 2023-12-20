// SPDX-License-Identifier: Apache-2.0

//! Resolve resource spans

use crate::attribute::resolve_attributes;
use crate::Error;
use weaver_schema::schema_spec::SchemaSpec;
use weaver_semconv::group::SpanKind;
use weaver_semconv::SemConvRegistry;
use weaver_version::VersionChanges;

/// Resolves resource spans in the given schema.
pub fn resolve_spans(
    schema: &mut SchemaSpec,
    sem_conv_catalog: &SemConvRegistry,
    version_changes: VersionChanges,
) -> Result<(), Error> {
    if let Some(spans) = schema.resource_spans.as_mut() {
        spans.attributes = resolve_attributes(
            spans.attributes.as_ref(),
            sem_conv_catalog,
            version_changes.span_attribute_changes(),
        )?;
        for span in spans.spans.iter_mut() {
            span.attributes = resolve_attributes(
                span.attributes.as_ref(),
                sem_conv_catalog,
                version_changes.span_attribute_changes(),
            )?;
            for event in span.events.iter_mut() {
                event.attributes = resolve_attributes(
                    event.attributes.as_ref(),
                    sem_conv_catalog,
                    version_changes.span_attribute_changes(),
                )?;
            }
            for link in span.links.iter_mut() {
                link.attributes = resolve_attributes(
                    link.attributes.as_ref(),
                    sem_conv_catalog,
                    version_changes.span_attribute_changes(),
                )?;
            }
        }
    }
    Ok(())
}

/// Resolve a span kind.
pub fn resolve_span_kind(span_kind: &SpanKind) -> weaver_resolved_schema::signal::SpanKind {
    match span_kind {
        SpanKind::Client => weaver_resolved_schema::signal::SpanKind::Client,
        SpanKind::Consumer => weaver_resolved_schema::signal::SpanKind::Consumer,
        SpanKind::Internal => weaver_resolved_schema::signal::SpanKind::Internal,
        SpanKind::Producer => weaver_resolved_schema::signal::SpanKind::Producer,
        SpanKind::Server => weaver_resolved_schema::signal::SpanKind::Server,
    }
}
