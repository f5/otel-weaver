// SPDX-License-Identifier: Apache-2.0

//! A schema specification.

use crate::instrumentation_library::InstrumentationLibrary;
use crate::resource::Resource;
use crate::resource_events::ResourceEvents;
use crate::resource_metrics::ResourceMetrics;
use crate::resource_spans::ResourceSpans;
use crate::tags::Tags;
use serde::{Deserialize, Serialize};

/// Definition of the telemetry schema for an application or a library.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
pub struct SchemaSpec {
    /// A set of tags for the schema.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Tags>,
    /// A common resource specification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<Resource>,
    /// The instrumentation library specification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instrumentation_library: Option<InstrumentationLibrary>,
    /// A resource metrics specification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_metrics: Option<ResourceMetrics>,
    /// A resource events specification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_events: Option<ResourceEvents>,
    /// A resource spans specification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_spans: Option<ResourceSpans>,
}
