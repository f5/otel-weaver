// SPDX-License-Identifier: Apache-2.0

//! Resource logs specification.

use crate::event::Event;
use semconv::attribute::Attribute;
use serde::{Deserialize, Serialize};
use crate::tags::Tags;

/// A resource events specification.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ResourceEvents {
    /// Common attributes shared across events (implemented as log records).
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<Attribute>,
    /// Definitions of structured events this application or library generates.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<Event>,
    /// A set of tags for the resource events.
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Tags>,
}
