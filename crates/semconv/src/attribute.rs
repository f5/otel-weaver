// SPDX-License-Identifier: Apache-2.0

//! Attribute specification.

use serde::{Deserialize, Serialize};

/// An attribute specification.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Attribute {
    /// The reference to the attribute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#ref: Option<String>,
    /// The id of the attribute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The type of the attribute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<AttributeType>,
    /// The brief of the attribute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brief: Option<String>,
    /// A collection of examples of the attribute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<Examples>,
    /// The note of the attribute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// An optional tag associated with the attribute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// The requirement level of the attribute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requirement_level: Option<RequirementLevel>,
    /// A flag indicating whether the attribute is relevant for sampling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling_relevant: Option<bool>,

    /// The value of the attribute.
    /// This is only used in a telemetry schema specification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
}

/// The different types of attributes.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum AttributeType {
    /// A basic attribute type.
    Basic(BasicAttributeType),
    /// A custom attribute type.
    Custom {
        /// A flag indicating whether custom values are allowed.
        #[serde(default)]
        allow_custom_values: bool,
        /// The members of the custom type.
        members: Vec<CustomTypeMember>,
    },
}

/// The different types of basic attributes.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum BasicAttributeType {
    /// A boolean attribute.
    Boolean,
    /// A integer attribute.
    Int,
    /// A double attribute.
    Double,
    /// A string attribute.
    String,
    /// Aa array of strings attribute.
    #[serde(rename = "string[]")]
    Strings,
}

/// A custom type member.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct CustomTypeMember {
    /// The id of the member.
    pub id: String,
    /// The value of the member.
    pub value: Value,
    /// The brief of the member.
    pub brief: Option<String>,
}

/// The different types of values.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum Value {
    /// A integer value.
    Int(i64),
    /// A double value.
    Double(f64),
    /// A string value.
    String(String),
}

/// The different types of examples.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum Examples {
    /// A integer example.
    Int(i64),
    /// A double example.
    Double(f64),
    /// A string example.
    String(String),
    /// A array of integers example.
    Ints(Vec<i64>),
    /// A array of doubles example.
    Doubles(Vec<f64>),
    /// A array of strings example.
    Strings(Vec<String>),
}

/// The different requirement levels.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum RequirementLevel {
    /// A basic requirement level.
    Basic(BasicRequirementLevel),
    /// A conditional requirement level.
    ConditionallyRequired {
        /// The description of the condition.
        #[serde(rename = "conditionally_required")]
        text: String,
    },
    /// A recommended requirement level.
    Recommended {
        /// The description of the recommendation.
        #[serde(rename = "recommended")]
        text: String,
    },
}

/// The different types of basic requirement levels.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum BasicRequirementLevel {
    /// A required requirement level.
    Required,
    /// An optional requirement level.
    Recommended,
    /// An opt-in requirement level.
    OptIn,
}
