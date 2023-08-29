use serde::{Deserialize, Serialize};
use crate::logs_version::LogsVersion;
use crate::metrics_version::MetricsVersion;
use crate::resource_version::ResourceVersion;
use crate::spans_version::SpansVersion;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct VersionSpec {
    pub metrics: Option<MetricsVersion>,
    pub logs: Option<LogsVersion>,
    pub spans: Option<SpansVersion>,
    pub resources: Option<ResourceVersion>,
}