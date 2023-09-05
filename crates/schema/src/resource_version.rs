//! Resource version.

use crate::resource_change::ResourceChange;
use serde::{Deserialize, Serialize};

/// Changes to apply to the resource for a specific version.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ResourceVersion {
    /// Changes to apply to the resource for a specific version.
    pub changes: Vec<ResourceChange>,
}
