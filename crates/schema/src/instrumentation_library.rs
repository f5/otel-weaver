use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct InstrumentationLibrary {
    pub name: Option<String>,
    pub version: Option<String>,
}