// SPDX-License-Identifier: Apache-2.0

//! Multivariate metrics.

use serde::{Deserialize, Serialize};

use semconv::attribute::Attribute;
use semconv::group::Instrument;

/// A multivariate metric specification.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct MultivariateMetrics {
    /// The name of the multivariate metric.
    pub id: String,
    /// The attributes of the multivariate metric.
    #[serde(default)]
    pub attributes: Vec<Attribute>,
    /// The metrics of the multivariate metric.
    #[serde(default)]
    pub metrics: Vec<Metric>,
}

/// A metric specification.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum Metric {
    /// A reference to a metric defined in a semantic convention catalog.
    Ref {
        /// The reference to the metric.
        r#ref: String
    },

    /// A fully defined metric.
    Metric{
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