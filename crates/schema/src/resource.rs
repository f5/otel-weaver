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
    pub attributes: Vec<Attribute>,
}
