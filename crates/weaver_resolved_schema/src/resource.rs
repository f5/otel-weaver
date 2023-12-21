// SPDX-License-Identifier: Apache-2.0

//! Define an OpenTelemetry resource.

use crate::attribute::AttributeRef;
use serde::{Deserialize, Serialize};

/// Definition of a resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resource {
    /// The attributes.
    pub attributes: Vec<AttributeRef>,
}
