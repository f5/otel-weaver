// SPDX-License-Identifier: Apache-2.0

//! Attribute resolution.

use std::collections::{BTreeMap, HashMap, HashSet};

use weaver_resolved_schema::attribute;
use weaver_resolved_schema::attribute::AttributeRef;
use weaver_schema::attribute::Attribute;
use weaver_schema::tags::Tags;
use weaver_semconv::attribute::{
    AttributeType, BasicRequirementLevel, Examples, PrimitiveOrArrayType, RequirementLevel,
    TemplateType, Value,
};
use weaver_semconv::group::ConvType;
use weaver_semconv::SemConvRegistry;
use weaver_version::VersionAttributeChanges;

use crate::{stability, Error};

/// A catalog of deduplicated resolved attributes with their corresponding reference.
#[derive(Debug, Default)]
pub struct AttributeCatalog {
    attributes: HashMap<attribute::Attribute, attribute::AttributeRef>,
}

impl AttributeCatalog {
    /// Returns the reference of the given attribute or creates a new reference if the attribute
    /// does not exist in the catalog.
    pub fn attribute_ref(&mut self, attr: attribute::Attribute) -> attribute::AttributeRef {
        let next_id = self.attributes.len() as u32;
        *self
            .attributes
            .entry(attr)
            .or_insert_with(|| attribute::AttributeRef(next_id))
    }

    /// Returns a list of deduplicated attributes ordered by their references.
    pub fn drain_attributes(self) -> Vec<attribute::Attribute> {
        let mut attributes: Vec<(attribute::Attribute, AttributeRef)> =
            self.attributes.into_iter().collect();
        attributes.sort_by_key(|(_, attr_ref)| attr_ref.0);
        attributes.into_iter().map(|(attr, _)| attr).collect()
    }
}

/// Resolves a collection of attributes (i.e. `Attribute::Ref`, `Attribute::AttributeGroupRef`,
/// and `Attribute::SpanRef`) from the given semantic convention catalog and local attributes
/// (i.e. `Attribute::Id`).
/// `Attribute::AttributeGroupRef` are first resolved, then `Attribute::SpanRef`, then
/// `Attribute::Ref`, and finally `Attribute::Id` are added.
/// An `Attribute::Ref` can override an attribute contained in an `Attribute::AttributeGroupRef`
/// or an `Attribute::SpanRef`.
/// An `Attribute::Id` can override an attribute contains in an `Attribute::Ref`, an
/// `Attribute::AttributeGroupRef`, or an `Attribute::SpanRef`.
///
/// Note: Version changes are used during the resolution process to determine the names of the
/// attributes.
pub fn resolve_attributes(
    attributes: &[Attribute],
    sem_conv_catalog: &weaver_semconv::SemConvRegistry,
    version_changes: impl VersionAttributeChanges,
) -> Result<Vec<Attribute>, Error> {
    let mut resolved_attrs = BTreeMap::new();
    let mut copy_into_resolved_attrs =
        |attrs: HashMap<&String, &weaver_semconv::attribute::Attribute>, tags: &Option<Tags>| {
            for (attr_id, attr) in attrs {
                let mut attr: Attribute = attr.into();
                attr.set_tags(tags);
                resolved_attrs.insert(attr_id.clone(), attr);
            }
        };

    // Resolve `Attribute::AttributeGroupRef`
    for attribute in attributes.iter() {
        if let Attribute::AttributeGroupRef {
            attribute_group_ref,
            tags,
        } = attribute
        {
            let attrs = sem_conv_catalog
                .attributes(attribute_group_ref, ConvType::AttributeGroup)
                .map_err(|e| Error::FailToResolveAttribute {
                    id: attribute_group_ref.clone(),
                    error: e.to_string(),
                })?;
            copy_into_resolved_attrs(attrs, tags);
        }
    }

    // Resolve `Attribute::ResourceRef`
    for attribute in attributes.iter() {
        if let Attribute::ResourceRef { resource_ref, tags } = attribute {
            let attrs = sem_conv_catalog
                .attributes(resource_ref, ConvType::Resource)
                .map_err(|e| Error::FailToResolveAttribute {
                    id: resource_ref.clone(),
                    error: e.to_string(),
                })?;
            copy_into_resolved_attrs(attrs, tags);
        }
    }

    // Resolve `Attribute::SpanRef`
    for attribute in attributes.iter() {
        if let Attribute::SpanRef { span_ref, tags } = attribute {
            let attrs = sem_conv_catalog
                .attributes(span_ref, ConvType::Span)
                .map_err(|e| Error::FailToResolveAttribute {
                    id: span_ref.clone(),
                    error: e.to_string(),
                })?;
            copy_into_resolved_attrs(attrs, tags);
        }
    }

    // Resolve `Attribute::EventRef`
    for attribute in attributes.iter() {
        if let Attribute::EventRef { event_ref, tags } = attribute {
            let attrs = sem_conv_catalog
                .attributes(event_ref, ConvType::Event)
                .map_err(|e| Error::FailToResolveAttribute {
                    id: event_ref.clone(),
                    error: e.to_string(),
                })?;
            copy_into_resolved_attrs(attrs, tags);
        }
    }

    // Resolve `Attribute::Ref`
    for attribute in attributes.iter() {
        if let Attribute::Ref { r#ref, .. } = attribute {
            let normalized_ref = version_changes.get_attribute_name(r#ref);
            let sem_conv_attr = sem_conv_catalog.attribute(&normalized_ref);
            let resolved_attribute = attribute.resolve_from(sem_conv_attr).map_err(|e| {
                Error::FailToResolveAttribute {
                    id: r#ref.clone(),
                    error: e.to_string(),
                }
            })?;
            resolved_attrs.insert(normalized_ref, resolved_attribute);
        }
    }

    // Resolve `Attribute::Id`
    // Note: any resolved attributes with the same id will be overridden.
    for attribute in attributes.iter() {
        if let Attribute::Id { id, .. } = attribute {
            resolved_attrs.insert(id.clone(), attribute.clone());
        }
    }

    Ok(resolved_attrs.into_values().collect())
}

/// Merges the given main attributes with the inherited attributes.
/// Main attributes have precedence over inherited attributes.
pub fn merge_attributes(main_attrs: &[Attribute], inherited_attrs: &[Attribute]) -> Vec<Attribute> {
    let mut merged_attrs = main_attrs.to_vec();
    let main_attr_ids = main_attrs
        .iter()
        .map(|attr| match attr {
            Attribute::Ref { r#ref, .. } => r#ref.clone(),
            Attribute::Id { id, .. } => id.clone(),
            Attribute::AttributeGroupRef { .. } => {
                panic!("Attribute groups are not supported yet")
            }
            Attribute::SpanRef { .. } => {
                panic!("Span references are not supported yet")
            }
            Attribute::ResourceRef { .. } => {
                panic!("Resource references are not supported yet")
            }
            Attribute::EventRef { .. } => {
                panic!("Event references are not supported yet")
            }
        })
        .collect::<HashSet<_>>();

    for inherited_attr in inherited_attrs.iter() {
        match inherited_attr {
            Attribute::Ref { r#ref, .. } => {
                if main_attr_ids.contains(r#ref) {
                    continue;
                }
            }
            Attribute::Id { id, .. } => {
                if main_attr_ids.contains(id) {
                    continue;
                }
            }
            Attribute::AttributeGroupRef { .. } => {
                panic!("Attribute groups are not supported yet")
            }
            Attribute::SpanRef { .. } => {
                panic!("Span references are not supported yet")
            }
            Attribute::ResourceRef { .. } => {
                panic!("Resource references are not supported yet")
            }
            Attribute::EventRef { .. } => {
                panic!("Event references are not supported yet")
            }
        }
        merged_attrs.push(inherited_attr.clone());
    }
    merged_attrs
}

/// Converts a semantic convention attribute to a resolved attribute.
pub fn resolve_attribute(
    registry: &SemConvRegistry,
    attr: &weaver_semconv::attribute::Attribute,
) -> Result<weaver_resolved_schema::attribute::Attribute, Error> {
    match attr {
        weaver_semconv::attribute::Attribute::Ref { r#ref, .. } => {
            let sem_conv_attr = registry
                .attribute(r#ref)
                .ok_or(Error::FailToResolveAttribute {
                    id: r#ref.clone(),
                    error: "Attribute ref not found in the resolved registry".to_string(),
                })?;
            resolve_attribute(registry, sem_conv_attr)
        }
        weaver_semconv::attribute::Attribute::Id {
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
        } => Ok(weaver_resolved_schema::attribute::Attribute {
            name: id.clone(),
            r#type: semconv_to_resolved_attr_type(r#type),
            brief: brief.clone(),
            examples: semconv_to_resolved_examples(examples),
            tag: tag.clone(),
            requirement_level: semconv_to_resolved_req_level(requirement_level),
            sampling_relevant: *sampling_relevant,
            note: note.clone(),
            stability: stability::resolve_stability(stability),
            deprecated: deprecated.clone(),
            tags: None,
            value: None,
        }),
    }
}

fn semconv_to_resolved_attr_type(
    attr_type: &AttributeType,
) -> weaver_resolved_schema::attribute::AttributeType {
    match attr_type {
        AttributeType::PrimitiveOrArray(poa) => match poa {
            PrimitiveOrArrayType::Boolean => {
                weaver_resolved_schema::attribute::AttributeType::Boolean
            }
            PrimitiveOrArrayType::Int => weaver_resolved_schema::attribute::AttributeType::Int,
            PrimitiveOrArrayType::Double => {
                weaver_resolved_schema::attribute::AttributeType::Double
            }
            PrimitiveOrArrayType::String => {
                weaver_resolved_schema::attribute::AttributeType::String
            }
            PrimitiveOrArrayType::Strings => {
                weaver_resolved_schema::attribute::AttributeType::Strings
            }
            PrimitiveOrArrayType::Ints => weaver_resolved_schema::attribute::AttributeType::Ints,
            PrimitiveOrArrayType::Doubles => {
                weaver_resolved_schema::attribute::AttributeType::Doubles
            }
            PrimitiveOrArrayType::Booleans => {
                weaver_resolved_schema::attribute::AttributeType::Booleans
            }
        },
        AttributeType::Template(template) => match template {
            TemplateType::Boolean => {
                weaver_resolved_schema::attribute::AttributeType::TemplateBoolean
            }
            TemplateType::Int => weaver_resolved_schema::attribute::AttributeType::TemplateInt,
            TemplateType::Double => {
                weaver_resolved_schema::attribute::AttributeType::TemplateDouble
            }
            TemplateType::String => {
                weaver_resolved_schema::attribute::AttributeType::TemplateString
            }
            TemplateType::Strings => {
                weaver_resolved_schema::attribute::AttributeType::TemplateStrings
            }
            TemplateType::Ints => weaver_resolved_schema::attribute::AttributeType::TemplateInts,
            TemplateType::Doubles => {
                weaver_resolved_schema::attribute::AttributeType::TemplateDoubles
            }
            TemplateType::Booleans => {
                weaver_resolved_schema::attribute::AttributeType::TemplateBooleans
            }
        },
        AttributeType::Enum {
            allow_custom_values,
            members,
        } => weaver_resolved_schema::attribute::AttributeType::Enum {
            allow_custom_values: *allow_custom_values,
            members: members
                .iter()
                .map(|member| weaver_resolved_schema::attribute::EnumEntries {
                    id: member.id.clone(),
                    value: match &member.value {
                        Value::String(s) => {
                            weaver_resolved_schema::value::Value::String { value: s.clone() }
                        }
                        Value::Int(i) => weaver_resolved_schema::value::Value::Int { value: *i },
                        Value::Double(d) => weaver_resolved_schema::value::Value::from_f64(*d),
                    },
                    brief: member.brief.clone(),
                    note: member.note.clone(),
                })
                .collect(),
        },
    }
}

fn semconv_to_resolved_examples(examples: &Option<Examples>) -> Option<attribute::Example> {
    examples.as_ref().map(|examples| match examples {
        Examples::Bool(v) => attribute::Example::Bool { value: *v },
        Examples::Int(v) => attribute::Example::Int { value: *v },
        Examples::Double(v) => attribute::Example::from_f64(*v),
        Examples::String(v) => attribute::Example::String { value: v.clone() },
        Examples::Ints(v) => weaver_resolved_schema::attribute::Example::Ints { values: v.clone() },
        Examples::Doubles(v) => attribute::Example::from_f64s(v.clone()),
        Examples::Bools(v) => attribute::Example::Bools { values: v.clone() },
        Examples::Strings(v) => attribute::Example::Strings { values: v.clone() },
    })
}

fn semconv_to_resolved_req_level(
    req_level: &RequirementLevel,
) -> weaver_resolved_schema::attribute::RequirementLevel {
    match req_level {
        RequirementLevel::Basic(level) => match level {
            BasicRequirementLevel::Required => {
                weaver_resolved_schema::attribute::RequirementLevel::Required
            }
            BasicRequirementLevel::Recommended => {
                weaver_resolved_schema::attribute::RequirementLevel::Recommended { text: None }
            }
            BasicRequirementLevel::OptIn => {
                weaver_resolved_schema::attribute::RequirementLevel::OptIn
            }
        },
        RequirementLevel::Recommended { text } => {
            weaver_resolved_schema::attribute::RequirementLevel::Recommended {
                text: Some(text.clone()),
            }
        }
        RequirementLevel::ConditionallyRequired { text } => {
            weaver_resolved_schema::attribute::RequirementLevel::ConditionallyRequired {
                text: text.clone(),
            }
        }
    }
}

#[allow(dead_code)] // ToDo Remove this once we have values in the resolved schema
fn semconv_to_resolved_value(
    value: &Option<Value>,
) -> Option<weaver_resolved_schema::value::Value> {
    value.as_ref().map(|value| match value {
        Value::String(s) => weaver_resolved_schema::value::Value::String { value: s.clone() },
        Value::Int(i) => weaver_resolved_schema::value::Value::Int { value: *i },
        Value::Double(d) => weaver_resolved_schema::value::Value::from_f64(*d),
    })
}
