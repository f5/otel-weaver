// SPDX-License-Identifier: Apache-2.0

//! Logs version.

use serde::{Deserialize, Serialize};
use crate::logs_change::LogsChange;

/// Changes to apply to the logs for a specific version.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct LogsVersion {
    /// Changes to apply to the logs for a specific version.
    pub changes: Vec<LogsChange>,
}
