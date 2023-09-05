// SPDX-License-Identifier: Apache-2.0

//! Instrumentation library specification.

use serde::{Deserialize, Serialize};

/// An instrumentation library specification.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct InstrumentationLibrary {
    /// An optional name for the instrumentation library.
    pub name: Option<String>,
    /// An optional version for the instrumentation library.
    pub version: Option<String>,
}
