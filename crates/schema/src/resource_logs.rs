//! Resource logs specification.

use crate::log::Log;
use semconv::attribute::Attribute;
use serde::{Deserialize, Serialize};

/// A resource logs specification.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ResourceLogs {
    /// The attributes of the resource logs.
    #[serde(default)]
    pub attributes: Vec<Attribute>,
    /// The logs of the resource logs.
    #[serde(default)]
    pub logs: Vec<Log>,
}
