// SPDX-License-Identifier: Apache-2.0

//! A resource metrics specification.

use crate::metric_group::MetricGroup;
use crate::univariate_metric::UnivariateMetric;
use crate::attribute::Attribute;
use serde::{Deserialize, Serialize};
use crate::tags::Tags;

/// A resource metrics specification.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
pub struct ResourceMetrics {
    /// Common attributes shared across metrics and metric groups.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<Attribute>,
    /// Definitions of all metrics this application or library generates (classic
    /// univariate OTel metrics).
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub metrics: Vec<UnivariateMetric>,
    /// Definitions of all multivariate metrics this application or library
    /// generates.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub metric_groups: Vec<MetricGroup>,
    /// A set of tags for the resource metrics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Tags>,
}
