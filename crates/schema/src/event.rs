// SPDX-License-Identifier: Apache-2.0

//! Log record specification.

use semconv::attribute::Attribute;
use serde::{Deserialize, Serialize};
use semconv::tags::Tags;

/// An event specification.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Event {
    /// The name of the event.
    pub event_name: String,
    /// The domain of the event.
    pub domain: String,
    /// The attributes of the log record.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<Attribute>,
    /// A set of tags for the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Tags>,
}