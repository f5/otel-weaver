// SPDX-License-Identifier: Apache-2.0

//! A univariate metric specification.

use semconv::attribute::Attribute;
use serde::{Deserialize, Serialize};
use semconv::group::Instrument;

/// A univariate metric specification.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum UnivariateMetric {
    /// A reference to a metric.
    Ref {
        /// The reference to the metric.
        r#ref: String,
        /// The attributes of the metric.
        #[serde(default)]
        #[serde(skip_serializing_if = "Vec::is_empty")]
        attributes: Vec<Attribute>,
    },

    /// A fully defined metric.
    Metric {
        /// Metric name.
        name: String,
        /// Brief description of the metric.
        brief: String,
        /// Note on the metric.
        note: String,
        /// Attributes of the metric.
        #[serde(default)]
        attributes: Vec<Attribute>,
        /// Type of the metric (e.g. gauge, histogram, ...).
        instrument: Option<Instrument>,
        /// Unit of the metric.
        unit: Option<String>,
    },
}
