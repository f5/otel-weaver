// SPDX-License-Identifier: Apache-2.0

//! Span specification.

use crate::span_event::SpanEvent;
use crate::span_link::SpanLink;
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
    pub events: Vec<SpanEvent>,
    /// The links of the span.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<SpanLink>,
}
