//! Event specification.

use semconv::attribute::Attribute;
use serde::{Deserialize, Serialize};

/// An event specification.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Event {
    /// The name of the event.
    pub id: String,
    /// The attributes of the event.
    #[serde(default)]
    pub attributes: Vec<Attribute>,
}
