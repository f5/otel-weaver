// SPDX-License-Identifier: Apache-2.0

//! Attribute specification.

use serde::{Deserialize, Serialize};
use crate::stability::Stability;

/// An attribute specification.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum Attribute {
    /// Reference to another attribute.
    Ref {
        /// The reference to the attribute.
        r#ref: String,
        /// The brief of the attribute.
        #[serde(skip_serializing_if = "Option::is_none")]
        brief: Option<String>,
        /// A collection of examples of the attribute.
        #[serde(skip_serializing_if = "Option::is_none")]
        examples: Option<Examples>,
        /// An optional tag associated with the attribute.
        #[serde(skip_serializing_if = "Option::is_none")]
        tag: Option<String>,
        /// The requirement level of the attribute.
        #[serde(skip_serializing_if = "Option::is_none")]
        requirement_level: Option<RequirementLevel>,
        /// A flag indicating whether the attribute is relevant for sampling.
        #[serde(skip_serializing_if = "Option::is_none")]
        sampling_relevant: Option<bool>,
        /// The note of the attribute.
        #[serde(skip_serializing_if = "Option::is_none")]
        note: Option<String>,
        /// Specifies the stability of the attribute.
        /// Note that, if stability is missing but deprecated is present, it will
        /// automatically set the stability to deprecated. If deprecated is
        /// present and stability differs from deprecated, this will result in an
        /// error.
        #[serde(skip_serializing_if = "Option::is_none")]
        stability: Option<Stability>,
        /// Specifies if the attribute is deprecated. The string
        /// provided as <description> MUST specify why it's deprecated and/or what
        /// to use instead. See also stability.
        #[serde(skip_serializing_if = "Option::is_none")]
        deprecated: Option<String>,

        /// The value of the attribute.
        /// This is only used in a telemetry schema specification.
        #[serde(skip_serializing_if = "Option::is_none")]
        value: Option<Value>,
    },
    /// Attribute definition.
    Id {
        /// The id of the attribute.
        id: String,
        /// The type of the attribute.
        r#type: AttributeType,
        /// The brief of the attribute.
        brief: String,
        /// A collection of examples of the attribute.
        examples: Option<Examples>,
        /// An optional tag associated with the attribute.
        #[serde(skip_serializing_if = "Option::is_none")]
        tag: Option<String>,
        /// The requirement level of the attribute.
        #[serde(skip_serializing_if = "Option::is_none")]
        requirement_level: Option<RequirementLevel>,
        /// A flag indicating whether the attribute is relevant for sampling.
        #[serde(skip_serializing_if = "Option::is_none")]
        sampling_relevant: Option<bool>,
        /// The note of the attribute.
        #[serde(skip_serializing_if = "Option::is_none")]
        note: Option<String>,
        /// Specifies the stability of the attribute.
        /// Note that, if stability is missing but deprecated is present, it will
        /// automatically set the stability to deprecated. If deprecated is
        /// present and stability differs from deprecated, this will result in an
        /// error.
        #[serde(skip_serializing_if = "Option::is_none")]
        stability: Option<Stability>,
        /// Specifies if the attribute is deprecated. The string
        /// provided as <description> MUST specify why it's deprecated and/or what
        /// to use instead. See also stability.
        #[serde(skip_serializing_if = "Option::is_none")]
        deprecated: Option<String>,

        /// The value of the attribute.
        /// This is only used in a telemetry schema specification.
        #[serde(skip_serializing_if = "Option::is_none")]
        value: Option<Value>,
    },
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
