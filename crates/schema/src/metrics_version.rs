use serde::{Deserialize, Serialize};
use crate::metrics_change::MetricsChange;
use crate::spans_change::SpansChange;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct MetricsVersion {
    pub changes: Vec<MetricsChange>,
}