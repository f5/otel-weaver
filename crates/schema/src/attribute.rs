// SPDX-License-Identifier: Apache-2.0

//! Definition of an attribute in the context of a telemetry schema.

use serde::{Deserialize, Serialize};

use semconv::attribute::{AttributeType, Examples, RequirementLevel, Value};
use semconv::stability::Stability;

use crate::tags::Tags;

/// An attribute specification.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
#[serde(rename_all = "snake_case")]
pub enum Attribute {
    /// Reference to another attribute.
    ///
    /// ref MUST have an id of an existing attribute.
    /// ref is useful for specifying that an existing attribute of another
    /// semantic convention is part of the current semantic convention and
    /// inherit its brief, note, and example values. However, if these fields
    /// are present in the current attribute definition, they override the
    /// inherited values.
    Ref {
        /// Reference an existing attribute.
        r#ref: String,
        /// A brief description of the attribute.
        #[serde(skip_serializing_if = "Option::is_none")]
        brief: Option<String>,
        /// Sequence of example values for the attribute or single example
        /// value. They are required only for string and string array
        /// attributes. Example values must be of the same type of the
        /// attribute. If only a single example is provided, it can directly
        /// be reported without encapsulating it into a sequence/dictionary.
        #[serde(skip_serializing_if = "Option::is_none")]
        examples: Option<Examples>,
        /// Associates a tag ("sub-group") to the attribute. It carries no
        /// particular semantic meaning but can be used e.g. for filtering
        /// in the markdown generator.
        #[serde(skip_serializing_if = "Option::is_none")]
        tag: Option<String>,
        /// Specifies if the attribute is mandatory. Can be "required",
        /// "conditionally_required", "recommended" or "opt_in". When omitted,
        /// the attribute is "recommended". When set to
        /// "conditionally_required", the string provided as <condition> MUST
        /// specify the conditions under which the attribute is required.
        #[serde(skip_serializing_if = "Option::is_none")]
        requirement_level: Option<RequirementLevel>,
        /// Specifies if the attribute is (especially) relevant for sampling
        /// and thus should be set at span start. It defaults to false.
        /// Note: this field is experimental.
        #[serde(skip_serializing_if = "Option::is_none")]
        sampling_relevant: Option<bool>,
        /// A more elaborate description of the attribute.
        /// It defaults to an empty string.
        #[serde(default)]
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
        /// A set of tags for the attribute.
        #[serde(skip_serializing_if = "Option::is_none")]
        tags: Option<Tags>,

        /// The value of the attribute.
        /// Note: This is only used in a telemetry schema specification.
        #[serde(skip_serializing_if = "Option::is_none")]
        value: Option<Value>,
    },
    /// Reference to an attribute group.
    ///
    /// `attribute_group_ref` MUST have an id of an existing attribute.
    AttributeGroupRef {
        /// Reference an existing attribute group.
        attribute_group_ref: String,
        /// A set of tags for the attribute.
        #[serde(skip_serializing_if = "Option::is_none")]
        tags: Option<Tags>,
    },
    /// Attribute definition.
    Id {
        /// String that uniquely identifies the attribute.
        id: String,
        /// Either a string literal denoting the type as a primitive or an
        /// array type, a template type or an enum definition.
        r#type: AttributeType,
        /// A brief description of the attribute.
        brief: String,
        /// Sequence of example values for the attribute or single example
        /// value. They are required only for string and string array
        /// attributes. Example values must be of the same type of the
        /// attribute. If only a single example is provided, it can directly
        /// be reported without encapsulating it into a sequence/dictionary.
        #[serde(skip_serializing_if = "Option::is_none")]
        examples: Option<Examples>,
        /// Associates a tag ("sub-group") to the attribute. It carries no
        /// particular semantic meaning but can be used e.g. for filtering
        /// in the markdown generator.
        #[serde(skip_serializing_if = "Option::is_none")]
        tag: Option<String>,
        /// Specifies if the attribute is mandatory. Can be "required",
        /// "conditionally_required", "recommended" or "opt_in". When omitted,
        /// the attribute is "recommended". When set to
        /// "conditionally_required", the string provided as <condition> MUST
        /// specify the conditions under which the attribute is required.
        #[serde(default)]
        requirement_level: RequirementLevel,
        /// Specifies if the attribute is (especially) relevant for sampling
        /// and thus should be set at span start. It defaults to false.
        /// Note: this field is experimental.
        #[serde(skip_serializing_if = "Option::is_none")]
        sampling_relevant: Option<bool>,
        /// A more elaborate description of the attribute.
        /// It defaults to an empty string.
        #[serde(default)]
        note: String,
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
        /// A set of tags for the attribute.
        #[serde(skip_serializing_if = "Option::is_none")]
        tags: Option<Tags>,

        /// The value of the attribute.
        /// Note: This is only used in a telemetry schema specification.
        #[serde(skip_serializing_if = "Option::is_none")]
        value: Option<Value>,
    },
}

impl From<&semconv::attribute::Attribute> for Attribute {
    /// Convert a semantic convention attribute to a schema attribute.
    fn from(attr: &semconv::attribute::Attribute) -> Self {
        match attr.clone() {
            semconv::attribute::Attribute::Ref {
                r#ref, brief,
                examples, tag,
                requirement_level, sampling_relevant,
                note, stability, deprecated
            } => Attribute::Ref {
                r#ref,
                brief,
                examples,
                tag,
                requirement_level,
                sampling_relevant,
                note,
                stability,
                deprecated,
                tags: None,
                value: None,
            },
            semconv::attribute::Attribute::Id {
                id, r#type, brief,
                examples, tag,
                requirement_level, sampling_relevant,
                note, stability, deprecated
            } => Attribute::Id {
                id,
                r#type,
                brief,
                examples,
                tag,
                requirement_level,
                sampling_relevant,
                note,
                stability,
                deprecated,
                tags: None,
                value: None,
            },
        }
    }
}

/// Convert a slice of semantic convention attributes to a vector of schema attributes.
pub fn from_semconv_attributes(attrs: &[semconv::attribute::Attribute]) -> Vec<Attribute> {
    attrs.iter().map(|attr| attr.into()).collect()
}
