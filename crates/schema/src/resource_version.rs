use serde::{Deserialize, Serialize};
use crate::resource_change::ResourceChange;
use crate::spans_change::SpansChange;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ResourceVersion {
    pub changes: Vec<ResourceChange>,
}