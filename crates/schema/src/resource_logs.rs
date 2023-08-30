use serde::{Deserialize, Serialize};
use semconv::attribute::Attribute;
use crate::log::Log;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ResourceLogs {
    #[serde(default)]
    pub attributes: Vec<Attribute>,
    #[serde(default)]
    pub logs: Vec<Log>,
}