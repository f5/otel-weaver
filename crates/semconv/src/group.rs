//! A group specification.

use crate::attribute::Attribute;
use serde::{Deserialize, Serialize};

/// A group specification.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Group {
    /// The id of the group.
    pub id: String,
    /// The type of the group.
    pub r#type: GroupType,
    /// The reference to the group this group extends.
    pub extends: Option<String>,
    /// The brief description of the group.
    pub brief: Option<String>,
    /// The prefix of the group.
    pub prefix: Option<String>,
    /// The note of the group.
    pub note: Option<String>,
    /// The attributes of the group.
    #[serde(default)]
    pub attributes: Vec<Attribute>,
    /// The constraints defined on the group.
    #[serde(default)]
    pub constraints: Vec<Constraint>,
    /// The span kind of the group.
    pub span_kind: Option<SpanKind>,
    /// The events of the group.
    #[serde(default)]
    pub events: Vec<String>,
    /// The metric name of the group.
    pub metric_name: Option<String>,
    /// The instrument of the group.
    pub instrument: Option<Instrument>,
    /// The unit of the group.
    pub unit: Option<String>,
}

/// The different types of groups.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum GroupType {
    /// A group of attributes.
    AttributeGroup,
    /// A group of spans.
    Span,
    /// A group of events.
    Event,
    /// A group of metrics.
    Metric,
}

/// The span kind.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SpanKind {
    /// A client span.
    Client,
    /// A server span.
    Server,
}

/// A constraint.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Constraint {
    /// A any_of constraint.
    #[serde(default)]
    pub any_of: Vec<String>,
    /// An include constraint.
    pub include: Option<String>,
}

/// The type of the metric.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Instrument {
    /// An up-down counter metric.
    #[serde(rename = "updowncounter")]
    UpDownCounter,
    /// A counter metric.
    Counter,
    /// A gauge metric.
    Gauge,
    /// A histogram metric.
    Histogram,
}
