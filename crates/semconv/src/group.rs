use serde::{Deserialize, Serialize};
use crate::attribute::Attribute;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Group {
    pub id: String,
    pub r#type: GroupType,
    pub extends: Option<String>,
    pub brief: Option<String>,
    pub prefix: Option<String>,
    pub note: Option<String>,
    #[serde(default)]
    pub attributes: Vec<Attribute>,
    #[serde(default)]
    pub constraints: Vec<Constraint>,
    pub span_kind: Option<SpanKind>,
    #[serde(default)]
    pub events: Vec<String>,
    pub metric_name: Option<String>,
    pub instrument: Option<Instrument>,
    pub unit: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum GroupType {
    AttributeGroup,
    Span,
    Event,
    Metric,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SpanKind {
    Client,
    Server,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Constraint {
    #[serde(default)]
    pub any_of: Vec<String>,
    pub include: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Instrument {
    #[serde(rename = "updowncounter")]
    UpDownCounter,
    Counter,
    Gauge,
    Histogram,
}