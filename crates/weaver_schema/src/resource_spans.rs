// SPDX-License-Identifier: Apache-2.0

//! A resource spans specification.

use crate::attribute::Attribute;
use crate::span::Span;
use crate::tags::Tags;
use serde::{Deserialize, Serialize};

/// A resource spans specification.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ResourceSpans {
    /// Common attributes shared across spans.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<Attribute>,
    /// Definitions of all spans this application or library generates.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub spans: Vec<Span>,
    /// A set of tags for the resource spans.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Tags>,
}
