use serde::{Deserialize, Serialize};
use semconv::attribute::Attribute;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum UnivariateMetric {
    Ref {
        r#ref: String,
        #[serde(default)]
        attributes: Vec<Attribute>,
    },
    Local {
        id: String,
        #[serde(default)]
        attributes: Vec<Attribute>,
    },
}