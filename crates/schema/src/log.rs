use serde::{Deserialize, Serialize};
use semconv::attribute::Attribute;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Log {
    pub id: String,
    pub body: BodyType,
    #[serde(default)]
    pub attributes: Vec<Attribute>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum BodyType {
    Boolean(bool),
    Int(i64),
    Double(f64),
    String(String),
    #[serde(rename = "boolean[]")]
    Booleans(Vec<String>),
    #[serde(rename = "int[]")]
    Ints(Vec<String>),
    #[serde(rename = "double[]")]
    Doubles(Vec<String>),
    #[serde(rename = "string[]")]
    Strings(Vec<String>),
}
