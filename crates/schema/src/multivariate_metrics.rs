// SPDX-License-Identifier: Apache-2.0

//! Multivariate metrics.

use serde::{Deserialize, Serialize};

use semconv::attribute::Attribute;

/// A multivariate metric specification.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct MultivariateMetrics {
    /// The name of the multivariate metric.
    id: String,
    /// The attributes of the multivariate metric.
    #[serde(default)]
    attributes: Vec<Attribute>,
    /// The metrics of the multivariate metric.
    #[serde(default)]
    metrics: Vec<Metric>,
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
    /// A metric defined locally.
    Local {
        /// The id of the metric.
        id: String
    },
}
