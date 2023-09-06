// SPDX-License-Identifier: Apache-2.0

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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<Attribute>,
    /// The logs of the resource logs.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub logs: Vec<Log>,
}
