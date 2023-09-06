// SPDX-License-Identifier: Apache-2.0

//! A schema specification.

use crate::instrumentation_library::InstrumentationLibrary;
use crate::resource::Resource;
use crate::resource_logs::ResourceLogs;
use crate::resource_metrics::ResourceMetrics;
use crate::resource_spans::ResourceSpans;
use serde::{Deserialize, Serialize};

/// A schema specification.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct SchemaSpec {
    /// A common resource specification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<Resource>,
    /// The instrumentation library specification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instrumentation_library: Option<InstrumentationLibrary>,
    /// A resource metrics specification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_metrics: Option<ResourceMetrics>,
    /// A resource logs specification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_logs: Option<ResourceLogs>,
    /// A resource spans specification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_spans: Option<ResourceSpans>,
}
