// SPDX-License-Identifier: Apache-2.0

//! Multivariate metrics.

use serde::{Deserialize, Serialize};

use semconv::attribute::Attribute;
use semconv::group::Instrument;
use semconv::tags::Tags;

/// The specification of a metric group.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct MetricGroup {
    /// The name of the metric group.
    pub id: String,
    /// The attributes of the metric group.
    #[serde(default)]
    pub attributes: Vec<Attribute>,
    /// The metrics of the metric group.
    #[serde(default)]
    pub metrics: Vec<Metric>,
    /// A set of tags for the metric group.
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Tags>,
}

/// A metric specification.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum Metric {
    /// A reference to a metric defined in a semantic convention catalog.
    Ref {
        /// The reference to the metric.
        r#ref: String,
        /// A set of tags for the metric group.
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