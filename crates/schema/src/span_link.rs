// SPDX-License-Identifier: Apache-2.0

//! Event specification.

use semconv::attribute::Attribute;
use serde::{Deserialize, Serialize};
use semconv::tags::Tags;

/// A span link specification.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct SpanLink {
    /// The name of the span link.
    pub id: String,
    /// The attributes of the span link.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<Attribute>,
    /// A set of tags for the span link.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Tags>,
}
