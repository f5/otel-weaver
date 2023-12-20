// SPDX-License-Identifier: Apache-2.0

//! Functions to resolve a semantic convention registry.

use weaver_logger::Logger;
use weaver_resolved_schema::registry::Registry;
use weaver_semconv::group::{ConvType, Group};
use weaver_semconv::SemConvRegistry;

use crate::constraint::resolve_constraints;
use crate::metrics::resolve_instrument;
use crate::spans::resolve_span_kind;
use crate::stability::resolve_stability;
use crate::Error;

/// Resolve a semantic convention registry.
pub fn resolve_semconv_registry(
    url: &str,
    registry: &SemConvRegistry,
    _log: impl Logger + Sync + Clone,
) -> Result<Registry, Error> {
    let groups = registry.groups().map(semconv_to_resolved_group).collect();

    Ok(Registry {
        registry_url: url.to_string(),
        groups,
    })
}

/// Resolve a semantic convention group.
fn semconv_to_resolved_group(group: &Group) -> weaver_resolved_schema::registry::Group {
    // ToDo resolve the attributes by identifying them in the catalog and create a list of AttributeRef.

    weaver_resolved_schema::registry::Group {
        id: group.id.clone(),
        typed_group: match group.r#type {
            ConvType::AttributeGroup => {
                weaver_resolved_schema::registry::TypedGroup::AttributeGroup {}
            }
            ConvType::Span => weaver_resolved_schema::registry::TypedGroup::Span {
                span_kind: group.span_kind.as_ref().map(resolve_span_kind),
                events: group.events.clone(),
            },
            ConvType::Event => weaver_resolved_schema::registry::TypedGroup::Event {
                name: group.name.clone(),
            },
            ConvType::Metric => weaver_resolved_schema::registry::TypedGroup::Metric {
                metric_name: group.metric_name.clone(),
                instrument: group.instrument.as_ref().map(resolve_instrument),
                unit: group.unit.clone(),
            },
            ConvType::MetricGroup => weaver_resolved_schema::registry::TypedGroup::MetricGroup {},
            ConvType::Resource => weaver_resolved_schema::registry::TypedGroup::Resource {},
            ConvType::Scope => weaver_resolved_schema::registry::TypedGroup::Scope {},
        },
        brief: group.brief.to_string(),
        note: group.note.to_string(),
        prefix: group.prefix.to_string(),
        extends: group.extends.clone(),
        stability: resolve_stability(&group.stability),
        deprecated: group.deprecated.clone(),
        constraints: resolve_constraints(&group.constraints),
        attributes: vec![],
    }
}
