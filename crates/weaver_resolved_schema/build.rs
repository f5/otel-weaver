// SPDX-License-Identifier: Apache-2.0

//! Build script for Weaver resolved schema.

use std::io::Result;
fn main() -> Result<()> {
    let mut config = prost_build::Config::new();
    config.btree_map(["."]);
    config.type_attribute(".", "#[derive(crate::Serialize, crate::Deserialize)]");

    config.compile_protos(&["proto/resolved-schema.proto"], &["proto/"])?;
    Ok(())
}
