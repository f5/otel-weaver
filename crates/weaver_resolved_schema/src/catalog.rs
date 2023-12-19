// SPDX-License-Identifier: Apache-2.0

#![allow(rustdoc::invalid_html_tags)]

//! Defines the catalog of attributes, metrics, and other telemetry items
//! that are shared across multiple signals in the Resolved Telemetry Schema.

use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::tags::Tags;

/// An internal reference to an attribute in the catalog.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AttributeRef(u32);

/// An internal reference to a metric in the catalog.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetricRef(u32);

/// A catalog of attributes, metrics, and other telemetry items that are shared
/// across multiple signals in the Resolved Telemetry Schema.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    /// Catalog of attributes used in the schema.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<Attribute>,
    /// Catalog of metrics used in the schema.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub metrics: Vec<Metric>,
}

/// An attribute definition.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Attribute {
    /// Attribute name.
    pub name: String,
    /// Either a string literal denoting the type as a primitive or an
    /// array type, a template type or an enum definition.
    pub r#type: AttributeType,
    /// A brief description of the attribute.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub brief: String,
    /// Sequence of example values for the attribute or single example
    /// value. They are required only for string and string array
    /// attributes. Example values must be of the same type of the
    /// attribute. If only a single example is provided, it can directly
    /// be reported without encapsulating it into a sequence/dictionary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<Examples>,
    /// Associates a tag ("sub-group") to the attribute. It carries no
    /// particular semantic meaning but can be used e.g. for filtering
    /// in the markdown generator.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// Specifies if the attribute is mandatory. Can be "required",
    /// "conditionally_required", "recommended" or "opt_in". When omitted,
    /// the attribute is "recommended". When set to
    /// "conditionally_required", the string provided as <condition> MUST
    /// specify the conditions under which the attribute is required.
    pub requirement_level: RequirementLevel,
    /// Specifies if the attribute is (especially) relevant for sampling
    /// and thus should be set at span start. It defaults to false.
    /// Note: this field is experimental.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling_relevant: Option<bool>,
    /// A more elaborate description of the attribute.
    /// It defaults to an empty string.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub note: String,
    /// Specifies the stability of the attribute.
    /// Note that, if stability is missing but deprecated is present, it will
    /// automatically set the stability to deprecated. If deprecated is
    /// present and stability differs from deprecated, this will result in an
    /// error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stability: Option<Stability>,
    /// Specifies if the attribute is deprecated. The string
    /// provided as <description> MUST specify why it's deprecated and/or what
    /// to use instead. See also stability.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<String>,
    /// A set of tags for the attribute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Tags>,

    /// The value of the attribute.
    /// Note: This is only used in a telemetry schema specification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
}

/// A metric definition.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Metric {
    /// Metric name.
    pub name: String,
    /// Brief description of the metric.
    pub brief: String,
    /// Note on the metric.
    pub note: String,
    /// Type of the metric (e.g. gauge, histogram, ...).
    pub instrument: Instrument,
    /// Unit of the metric.
    pub unit: Option<String>,
    /// A set of tags for the metric.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Tags>,
}

/// The different types of attributes.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum AttributeType {
    /// A boolean attribute.
    Boolean,
    /// A integer attribute (signed 64 bit integer).
    Int,
    /// A double attribute (double precision floating point (IEEE 754-1985)).
    Double,
    /// A string attribute.
    String,
    /// An array of strings attribute.
    Strings,
    /// An array of integer attribute.
    Ints,
    /// An array of double attribute.
    Doubles,
    /// An array of boolean attribute.
    Booleans,

    /// A template boolean attribute.
    TemplateBoolean,
    /// A template integer attribute.
    #[serde(rename = "template[int]")]
    TemplateInt,
    /// A template double attribute.
    #[serde(rename = "template[double]")]
    TemplateDouble,
    /// A template string attribute.
    #[serde(rename = "template[string]")]
    TemplateString,
    /// A template array of strings attribute.
    #[serde(rename = "template[string[]]")]
    TemplateStrings,
    /// A template array of integer attribute.
    #[serde(rename = "template[int[]]")]
    TemplateInts,
    /// A template array of double attribute.
    #[serde(rename = "template[double[]]")]
    TemplateDoubles,
    /// A template array of boolean attribute.
    #[serde(rename = "template[boolean[]]")]
    TemplateBooleans,

    /// An enum definition type.
    Enum {
        /// Set to false to not accept values other than the specified members.
        /// It defaults to true.
        allow_custom_values: bool,
        /// List of enum entries.
        members: Vec<EnumEntries>,
    },
}

/// Possible enum entries.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EnumEntries {
    /// String that uniquely identifies the enum entry.
    pub id: String,
    /// String, int, or boolean; value of the enum entry.
    pub value: Value,
    /// Brief description of the enum entry value.
    /// It defaults to the value of id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brief: Option<String>,
    /// Longer description.
    /// It defaults to an empty string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// The different types of examples.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Examples {
    /// A boolean example.
    Bool {
        /// The value of the example.
        value: bool,
    },
    /// A integer example.
    Int {
        /// The value of the example.
        value: i64,
    },
    /// A double example.
    Double {
        /// The value of the example.
        value: f64,
    },
    /// A string example.
    String {
        /// The value of the example.
        value: String,
    },
    /// A array of integers example.
    Ints {
        /// The value of the example.
        values: Vec<i64>,
    },
    /// A array of doubles example.
    Doubles {
        /// The value of the example.
        values: Vec<f64>,
    },
    /// A array of bools example.
    Bools {
        /// The value of the example.
        values: Vec<bool>,
    },
    /// A array of strings example.
    Strings {
        /// The value of the example.
        values: Vec<String>,
    },
}

/// The different requirement levels.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum RequirementLevel {
    /// A required requirement level.
    Required,
    /// An optional requirement level.
    Recommended {
        /// The description of the recommendation.
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<String>,
    },
    /// An opt-in requirement level.
    OptIn,
    /// A conditional requirement level.
    ConditionallyRequired {
        /// The description of the condition.
        #[serde(skip_serializing_if = "String::is_empty")]
        text: String,
    },
}

/// The level of stability for a definition.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Stability {
    /// A deprecated definition.
    Deprecated,
    /// An experimental definition.
    Experimental,
    /// A stable definition.
    Stable,
}

/// The different types of values.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum Value {
    /// A integer value.
    Int {
        /// The value
        value: i64,
    },
    /// A double value.
    Double {
        /// The value
        value: f64,
    },
    /// A string value.
    String {
        /// The value
        value: String,
    },
}

/// The type of the metric.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Instrument {
    /// An up-down counter metric.
    UpDownCounter,
    /// A counter metric.
    Counter,
    /// A gauge metric.
    Gauge,
    /// A histogram metric.
    Histogram,
}
