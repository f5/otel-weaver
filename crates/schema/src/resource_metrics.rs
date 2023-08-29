use serde::{Deserialize, Serialize};
use semconv::attribute::Attribute;
use crate::multivariate_metrics::MultivariateMetrics;
use crate::univariate_metric::UnivariateMetric;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ResourceMetrics {
    #[serde(default)]
    pub attributes: Vec<Attribute>,
    #[serde(default)]
    pub univariate_metrics: Vec<UnivariateMetric>,
    #[serde(default)]
    pub multivariate_metrics: Vec<MultivariateMetrics>,
}