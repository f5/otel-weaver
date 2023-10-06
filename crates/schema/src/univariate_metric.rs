// SPDX-License-Identifier: Apache-2.0

//! A univariate metric specification.

use crate::attribute::Attribute;
use crate::tags::Tags;
use semconv::group::Instrument;
use serde::{Deserialize, Serialize};

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
        /// A set of tags for the metric.
        #[serde(skip_serializing_if = "Option::is_none")]
        tags: Option<Tags>,
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
        /// A set of tags for the metric.
        #[serde(skip_serializing_if = "Option::is_none")]
        tags: Option<Tags>,
    },
}
