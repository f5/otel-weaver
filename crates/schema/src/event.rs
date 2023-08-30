use serde::{Deserialize, Serialize};
use semconv::attribute::Attribute;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub id: String,
    #[serde(default)]
    pub attributes: Vec<Attribute>,
}