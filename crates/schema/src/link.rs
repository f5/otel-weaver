// SPDX-License-Identifier: Apache-2.0

//! Event specification.

use semconv::attribute::Attribute;
use serde::{Deserialize, Serialize};

/// A link specification.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Link {
    /// The name of the link.
    pub id: String,
    /// The attributes of the link.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<Attribute>,
}
