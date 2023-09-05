//! Metrics change definitions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Changes to apply to the metrics for a specific version.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct MetricsChange {
    /// A collection of rename operations to apply to the metric names.
    pub rename_metrics: HashMap<String, String>,
}
