//! Changes to apply to the spans specification for a specific version.

use crate::spans_change::SpansChange;
use serde::{Deserialize, Serialize};

/// Changes to apply to the spans specification for a specific version.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct SpansVersion {
    /// Changes to apply to the spans specification for a specific version.
    pub changes: Vec<SpansChange>,
}
