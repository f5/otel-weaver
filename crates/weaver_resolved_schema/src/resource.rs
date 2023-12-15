// SPDX-License-Identifier: Apache-2.0

//! Define an OpenTelemetry resource.

use crate::catalog::AttributeRef;
use serde::{Deserialize, Serialize};

/// Definition of a resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resource {
    /// The attributes.
    attributes: Vec<AttributeRef>,
}
