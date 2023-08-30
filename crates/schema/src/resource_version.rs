use serde::{Deserialize, Serialize};
use crate::resource_change::ResourceChange;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ResourceVersion {
    pub changes: Vec<ResourceChange>,
}