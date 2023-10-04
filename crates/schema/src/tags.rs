// SPDX-License-Identifier: Apache-2.0

//! Tags for telemetry schemas.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// A set of tags.
#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
#[serde(deny_unknown_fields)]
pub struct Tags {
    /// The tags.
    tags: HashMap<String, String>
}

impl Tags {
    /// Checks if the tags contain a specific tag.
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains_key(tag)
    }

    /// Gets a specific tag value from the tags if it exists or `None` otherwise.
    pub fn get_tag(&self, tag: &str) -> Option<&String> {
        self.tags.get(tag)
    }

    /// Gets an iterator over the tags.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.tags.iter()
    }
}