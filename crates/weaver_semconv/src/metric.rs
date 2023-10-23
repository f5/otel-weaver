// SPDX-License-Identifier: Apache-2.0

//! Metric specification.

use crate::attribute::Attribute;
use crate::group::Instrument;
use serde::{Deserialize, Serialize};

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
    /// Set of attribute ids attached to the metric.
    #[serde(default)]
    pub attributes: Vec<Attribute>,
    /// Type of the metric (e.g. gauge, histogram, ...).
    pub instrument: Instrument,
    /// Unit of the metric.
    pub unit: Option<String>,
}

impl Metric {
    /// Returns the name of the metric.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the brief description of the metric.
    pub fn brief(&self) -> &str {
        &self.brief
    }

    /// Returns the note on the metric.
    pub fn note(&self) -> &str {
        &self.note
    }

    /// Returns the unit of the metric.
    pub fn unit(&self) -> Option<&str> {
        self.unit.as_deref()
    }
}
