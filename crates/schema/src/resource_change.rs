use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ResourceChange {
    pub rename_attributes: RenameAttributes,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct RenameAttributes {
    pub attribute_map: HashMap<String, String>,
}

