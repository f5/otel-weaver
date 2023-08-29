use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Attribute {
    pub r#ref: Option<String>,
    pub id: Option<String>,
    pub r#type: Option<AttributeType>,
    pub brief: Option<String>,
    pub examples: Option<Examples>,
    pub note: Option<String>,
    pub tag: Option<String>,
    pub requirement_level: Option<RequirementLevel>,
    pub sampling_relevant: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum AttributeType {
    Basic(BasicAttributeType),
    Custom {
        #[serde(default)]
        allow_custom_values: bool,
        members: Vec<CustomTypeMember>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum BasicAttributeType {
    Boolean,
    Int,
    Double,
    String,
    #[serde(rename = "string[]")]
    Strings,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct CustomTypeMember {
    pub id: String,
    pub value: Value,
    pub brief: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum Value {
    Int(i64),
    Double(f64),
    String(String),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum Examples {
    Int(i64),
    Double(f64),
    String(String),
    Ints(Vec<i64>),
    Doubles(Vec<f64>),
    Strings(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum RequirementLevel {
    Basic(BasicRequirementLevel),
    ConditionallyRequired{
        #[serde(rename = "conditionally_required")]
        text: String
    },
    Recommended{
        #[serde(rename = "recommended")]
        text: String
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum BasicRequirementLevel {
    Required,
    Recommended,
    OptIn,
}