// SPDX-License-Identifier: Apache-2.0

//! Span specification.

use crate::span_event::SpanEvent;
use crate::span_link::SpanLink;
use crate::attribute::Attribute;
use serde::{Deserialize, Serialize};
use semconv::group::SpanKind;
use crate::tags::Tags;

/// A span specification.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
pub struct Span {
    /// The name of the span.
    pub span_name: String,
    /// The kind of the span.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<SpanKind>,
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
    /// A set of tags for the span.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Tags>,
}
