// SPDX-License-Identifier: Apache-2.0

//! Defines the catalog of attributes, metrics, and other telemetry items
//! that are shared across multiple signals in the Resolved Telemetry Schema.

use crate::attribute::Attribute;
use crate::metric::Metric;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// A catalog of attributes, metrics, and other telemetry signals that are shared
/// in the Resolved Telemetry Schema.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    /// Catalog of attributes used in the schema.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<Attribute>,
    /// Catalog of metrics used in the schema.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub metrics: Vec<Metric>,
}

/// The level of stability for a definition.
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Stability {
    /// A deprecated definition.
    Deprecated,
    /// An experimental definition.
    Experimental,
    /// A stable definition.
    Stable,
}
