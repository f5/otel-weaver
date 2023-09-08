// SPDX-License-Identifier: Apache-2.0

//! A group specification.

use crate::attribute::Attribute;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};
use crate::stability::Stability;

/// Groups contain the list of semantic conventions and it is the root node of
/// each yaml file.
#[derive(Serialize, Deserialize, Debug, Validate)]
#[serde(deny_unknown_fields)]
#[validate(schema(function = "validate_group"))]
pub struct Group {
    /// The id that uniquely identifies the semantic convention.
    pub id: String,
    /// The type of the semantic convention (default to span).
    #[serde(default)]
    pub r#type: ConvType,
    /// A brief description of the semantic convention.
    pub brief: String,
    /// A more elaborate description of the semantic convention.
    /// It defaults to an empty string.
    #[serde(default)]
    pub note: String,
    /// Prefix for the attributes for this semantic convention.
    /// It defaults to an empty string.
    #[serde(default)]
    pub prefix: String,
    /// Reference another semantic convention id. It inherits the prefix,
    /// constraints, and all attributes defined in the specified semantic
    /// convention.
    pub extends: Option<String>,
    /// Specifies the stability of the semantic convention.
    /// Note that, if stability is missing but deprecated is present, it will
    /// automatically set the stability to deprecated. If deprecated is
    /// present and stability differs from deprecated, this will result in an
    /// error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stability: Option<Stability>,
    /// Specifies if the semantic convention is deprecated. The string
    /// provided as <description> MUST specify why it's deprecated and/or what
    /// to use instead. See also stability.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<String>,
    /// List of attributes that belong to the semantic convention.
    #[serde(default)]
    pub attributes: Vec<Attribute>,
    /// Additional constraints. It defaults to an empty list.
    #[serde(default)]
    pub constraints: Vec<Constraint>,
    /// Specifies the kind of the span.
    /// Note: only valid if type is span (the default)
    pub span_kind: Option<SpanKind>,
    /// List of strings that specify the ids of event semantic conventions
    /// associated with this span semantic convention.
    /// Note: only valid if type is span (the default)
    #[serde(default)]
    pub events: Vec<String>,
    /// The metric name as described by the [OpenTelemetry Specification](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/data-model.md#timeseries-model).
    /// Note: This field is required if type is metric.
    pub metric_name: Option<String>,
    /// The instrument type that should be used to record the metric. Note that
    /// the semantic conventions must be written using the names of the
    /// synchronous instrument types (counter, gauge, updowncounter and
    /// histogram).
    /// For more details: [Metrics semantic conventions - Instrument types](https://github.com/open-telemetry/opentelemetry-specification/tree/main/specification/metrics/semantic_conventions#instrument-types).
    /// Note: This field is required if type is metric.
    pub instrument: Option<Instrument>,
    /// The unit in which the metric is measured, which should adhere to the
    /// [guidelines](https://github.com/open-telemetry/opentelemetry-specification/tree/main/specification/metrics/semantic_conventions#instrument-units).
    /// Note: This field is required if type is metric.
    pub unit: Option<String>,
    /// The name of the event. If not specified, the prefix is used.
    /// If prefix is empty (or unspecified), name is required.
    pub name: Option<String>,
}

/// Validation logic for the group.
fn validate_group(group: &Group) -> Result<(), ValidationError> {
    // If deprecated is present and stability differs from deprecated, this
    // will result in an error.
    if group.deprecated.is_some() && group.stability.is_some() {
        if group.stability != Some(Stability::Deprecated) {
            return Err(ValidationError::new("This group contains a deprecated field but the stability is not set to deprecated."));
        }
    }

    // Fields span_kind and events are only valid if type is span (the default).
    if group.r#type != ConvType::Span {
        if group.span_kind.is_some() {
            return Err(ValidationError::new("This group contains a span_kind field but the type is not set to span."));
        }
        if !group.events.is_empty() {
            return Err(ValidationError::new("This group contains an events field but the type is not set to span."));
        }
    }

    // Field name is required if prefix is empty and if type is event.
    if group.r#type == ConvType::Event {
        if group.prefix.is_empty() && group.name.is_none() {
            return Err(ValidationError::new("This group contains an event type but the prefix is empty and the name is not set."));
        }
    }

    // Fields metric_name, instrument and unit are required if type is metric.
    if group.r#type == ConvType::Metric {
        if group.metric_name.is_none() {
            return Err(ValidationError::new("This group contains a metric type but the metric_name is not set."));
        }
        if group.instrument.is_none() {
            return Err(ValidationError::new("This group contains a metric type but the instrument is not set."));
        }
        if group.unit.is_none() {
            return Err(ValidationError::new("This group contains a metric type but the unit is not set."));
        }
    }

    println!("ToDo Attribute validation");

    Ok(())
}

/// The different types of groups.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConvType {
    /// Attribute group (attribute_group type) defines a set of attributes that
    /// can be declared once and referenced by semantic conventions for
    /// different signals, for example spans and logs. Attribute groups don't
    /// have any specific fields and follow the general semconv semantics.
    AttributeGroup,
    /// Span semantic convention.
    Span,
    /// Event semantic convention.
    Event,
    /// Metric semantic convention.
    Metric,
    /// The metric group semconv is a group where related metric attributes can
    /// be defined and then referenced from other metric groups using ref.
    MetricGroup,
    /// A group of resources.
    Resource,
    /// Scope.
    Scope,
}

impl Default for ConvType {
    /// Returns the default convention type that is span based on
    /// the OpenTelemetry specification.
    fn default() -> Self {
        Self::Span
    }
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
#[derive(Serialize, Deserialize, Debug, Clone)]
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
