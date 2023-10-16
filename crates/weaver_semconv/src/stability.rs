// SPDX-License-Identifier: Apache-2.0

//! Stability specification.

use serde::{Deserialize, Serialize};

/// The level of stability for a definition.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Stability {
    /// A deprecated definition.
    Deprecated,
    /// An experimental definition.
    Experimental,
    /// A stable definition.
    Stable,
}
