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
    pub attributes: Vec<Attribute>,
    /// The spans of the resource spans.
    #[serde(default)]
    pub spans: Vec<Span>,
}
