// SPDX-License-Identifier: Apache-2.0

//! Metric specification.

use serde::{Deserialize, Serialize};
use crate::attribute::Attribute;
use crate::group::Instrument;

/// A metric specification.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Metric {
    /// Metric name.
    pub name: String,
    /// Brief description of the metric.
    pub brief: String,
    /// Note on the metric.
    pub note: String,
    /// Attributes of the metric.
    #[serde(default)]
    pub attributes: Vec<Attribute>,
    /// Type of the metric (e.g. gauge, histogram, ...).
    pub instrument: Option<Instrument>,
    /// Unit of the metric.
    pub unit: Option<String>,
}
