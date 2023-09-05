//! A univariate metric specification.

use semconv::attribute::Attribute;
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
        attributes: Vec<Attribute>,
    },
    /// A local metric.
    Local {
        /// The id of the metric.
        id: String,
        /// The attributes of the metric.
        #[serde(default)]
        attributes: Vec<Attribute>,
    },
}
