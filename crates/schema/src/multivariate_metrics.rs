use serde::{Deserialize, Serialize};

use semconv::attribute::Attribute;
use crate::univariate_metric::UnivariateMetric;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct MultivariateMetrics {
    id: String,
    #[serde(default)]
    attributes: Vec<Attribute>,
    #[serde(default)]
    metrics: Vec<Metric>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum Metric {
    Ref {
        r#ref: String,
    },
    Local {
        id: String,
    },
}