//! The specification of the changes to apply to the schema for different versions.

use crate::logs_version::LogsVersion;
use crate::metrics_version::MetricsVersion;
use crate::resource_version::ResourceVersion;
use crate::spans_version::SpansVersion;
use serde::{Deserialize, Serialize};

/// An history of changes to apply to the schema for different versions.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct VersionSpec {
    /// The changes to apply to the metrics specification for a specific version.
    pub metrics: Option<MetricsVersion>,
    /// The changes to apply to the logs specification for a specific version.
    pub logs: Option<LogsVersion>,
    /// The changes to apply to the spans specification for a specific version.
    pub spans: Option<SpansVersion>,
    /// The changes to apply to the resource specification for a specific version.
    pub resources: Option<ResourceVersion>,
}
