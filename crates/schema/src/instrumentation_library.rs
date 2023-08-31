use serde::{Deserialize, Serialize};
use crate::resource::Resource;
use crate::resource_logs::ResourceLogs;
use crate::resource_metrics::ResourceMetrics;
use crate::resource_spans::ResourceSpans;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct InstrumentationLibrary {
    pub name: Option<String>,
    pub version: Option<String>,
}