// SPDX-License-Identifier: Apache-2.0

//! A resource metrics specification.

use crate::multivariate_metrics::MultivariateMetrics;
use crate::univariate_metric::UnivariateMetric;
use semconv::attribute::Attribute;
use serde::{Deserialize, Serialize};

/// A resource metrics specification.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ResourceMetrics {
    /// The attributes of the resource metrics.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<Attribute>,
    /// The univariate metrics of the resource metrics.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub univariate_metrics: Vec<UnivariateMetric>,
    /// The multivariate metrics of the resource metrics.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub multivariate_metrics: Vec<MultivariateMetrics>,
}
