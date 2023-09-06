// SPDX-License-Identifier: Apache-2.0

//! Span specification.

use crate::event::Event;
use crate::link::Link;
use semconv::attribute::Attribute;
use serde::{Deserialize, Serialize};

/// A span specification.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Span {
    /// The id of the span.
    pub id: String,
    /// The attributes of the span.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<Attribute>,
    /// The events of the span.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<Event>,
    /// The links of the span.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<Link>,
}
