// SPDX-License-Identifier: Apache-2.0

//! A resource spans specification.

use crate::span::Span;
use semconv::attribute::Attribute;
use serde::{Deserialize, Serialize};

/// A resource spans specification.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ResourceSpans {
    /// The attributes of the resource spans.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<Attribute>,
    /// The spans of the resource spans.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub spans: Vec<Span>,
}
