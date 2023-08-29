use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Attribute {
    pub r#ref: Option<String>,
    pub value: Option<String>, // TODO
    pub id: Option<String>,
    pub r#type: Option<String>,
    pub brief: Option<String>,
    pub tag: Option<String>,
    pub requirement_level: Option<RequirementLevel>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum RequirementLevel {
    Required,
    Optional,
    Recommended,
}