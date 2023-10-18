// SPDX-License-Identifier: Apache-2.0

//! Command to generate a client API (third party)

use std::path::Path;
use weaver_logger::{ILogger};

/// Generate a client API (third party)
pub fn command_gen_client_api(_log: impl ILogger + Sync + Clone, _schema: &Path) {}
