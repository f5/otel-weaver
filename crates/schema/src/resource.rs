// SPDX-License-Identifier: Apache-2.0

//! A common resource specification.

use semconv::attribute::Attribute;
use serde::{Deserialize, Serialize};

/// A common resource specification.
/// All the attributes mentioned in this specification will be inherited by all
/// the other specialized resource specifications.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Resource {
    /// The common attributes of the resource.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<Attribute>,
}
