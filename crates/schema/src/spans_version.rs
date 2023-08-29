use serde::{Deserialize, Serialize};
use crate::spans_change::SpansChange;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct SpansVersion {
    pub changes: Vec<SpansChange>,
}