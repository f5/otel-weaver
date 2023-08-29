use serde::{Deserialize, Serialize};
use crate::attribute::Attribute;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ResourceMetrics {
    #[serde(default)]
    pub attributes: Vec<Attribute>,
}