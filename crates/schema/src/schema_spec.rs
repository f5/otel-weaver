use serde::{Deserialize, Serialize};
use crate::instrumentation_library::InstrumentationLibrary;
use crate::resource::Resource;
use crate::resource_logs::ResourceLogs;
use crate::resource_metrics::ResourceMetrics;
use crate::resource_spans::ResourceSpans;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct SchemaSpec {
    pub resource: Option<Resource>,
    pub instrumentation_library: Option<InstrumentationLibrary>,
    pub resource_metrics: Option<ResourceMetrics>,
    pub resource_logs: Option<ResourceLogs>,
    pub resource_spans: Option<ResourceSpans>,
}