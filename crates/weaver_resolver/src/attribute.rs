// SPDX-License-Identifier: Apache-2.0

//! Attribute resolution.

use serde::Deserialize;
use std::collections::{BTreeMap, HashMap, HashSet};

use weaver_resolved_schema::attribute;
use weaver_resolved_schema::attribute::AttributeRef;
use weaver_schema::attribute::Attribute;
use weaver_schema::tags::Tags;
use weaver_semconv::attribute::{
    AttributeSpec, AttributeTypeSpec, BasicRequirementLevel, Examples, PrimitiveOrArrayType,
    RequirementLevelSpec, TemplateType, Value,
};
use weaver_semconv::group::ConvTypeSpec;
use weaver_semconv::SemConvSpecs;
use weaver_version::VersionAttributeChanges;

use crate::{stability, Error};

/// A catalog of deduplicated resolved attributes with their corresponding reference.
#[derive(Deserialize, Debug, Default, PartialEq)]
pub struct AttributeCatalog {
    /// A map of deduplicated resolved attributes with their corresponding reference.
    attribute_refs: HashMap<attribute::Attribute, AttributeRef>,
    #[serde(skip)]
    /// A map of root attributes indexed by their name.
    /// Root attributes are attributes that doesn't inherit from another attribute.
    root_attributes: HashMap<String, attribute::Attribute>,
}

impl AttributeCatalog {
    /// Returns the reference of the given attribute or creates a new reference if the attribute
    /// does not exist in the catalog.
    pub fn attribute_ref(&mut self, attr: attribute::Attribute) -> AttributeRef {
        let next_id = self.attribute_refs.len() as u32;
        *self
            .attribute_refs
            .entry(attr)
            .or_insert_with(|| AttributeRef(next_id))
    }

    /// Returns a list of deduplicated attributes ordered by their references.
    pub fn drain_attributes(self) -> Vec<attribute::Attribute> {
        let mut attributes: Vec<(attribute::Attribute, AttributeRef)> =
            self.attribute_refs.into_iter().collect();
        attributes.sort_by_key(|(_, attr_ref)| attr_ref.0);
        attributes.into_iter().map(|(attr, _)| attr).collect()
    }

    /// Tries to resolve the given attribute spec (ref or id) from the catalog.
    /// Returns `None` if the attribute spec is a ref and it does not exist yet
    /// in the catalog.
    pub fn resolve(&mut self, prefix: &str, attr: &AttributeSpec) -> Option<AttributeRef> {
        match attr {
            AttributeSpec::Ref {
                r#ref,
                brief,
                examples,
                tag,
                requirement_level,
                sampling_relevant,
                note,
                stability,
                deprecated,
            } => {
                let root_attr = self.root_attributes.get(r#ref);
                if let Some(root_attr) = root_attr {
                    // Create a fully resolved attribute from an attribute spec
                    // (ref) and override the root attribute with the new
                    // values if they are present.
                    let resolved_attr = attribute::Attribute {
                        name: r#ref.clone(),
                        r#type: root_attr.r#type.clone(),
                        brief: match brief {
                            Some(brief) => brief.clone(),
                            None => root_attr.brief.clone(),
                        },
                        examples: match examples {
                            Some(_) => semconv_to_resolved_examples(examples),
                            None => root_attr.examples.clone(),
                        },
                        tag: match tag {
                            Some(_) => tag.clone(),
                            None => root_attr.tag.clone(),
                        },
                        requirement_level: match requirement_level {
                            Some(requirement_level) => {
                                semconv_to_resolved_req_level(requirement_level)
                            }
                            None => root_attr.requirement_level.clone(),
                        },
                        sampling_relevant: match sampling_relevant {
                            Some(_) => *sampling_relevant,
                            None => root_attr.sampling_relevant,
                        },
                        note: match note {
                            Some(note) => note.clone(),
                            None => root_attr.note.clone(),
                        },
                        stability: match stability {
                            Some(_) => stability::resolve_stability(stability),
                            None => root_attr.stability.clone(),
                        },
                        deprecated: match deprecated {
                            Some(_) => deprecated.clone(),
                            None => root_attr.deprecated.clone(),
                        },
                        tags: root_attr.tags.clone(),
                        value: root_attr.value.clone(),
                    };
                    Some(self.attribute_ref(resolved_attr))
                } else {
                    None
                }
            }
            AttributeSpec::Id {
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
            } => {
                let root_attr_id = if prefix.is_empty() {
                    id.clone()
                } else {
                    format!("{}.{}", prefix, id)
                };

                // Create a fully resolved attribute from an attribute spec (id),
                // and check if it already exists in the catalog.
                // If it does, return the reference to the existing attribute.
                // If it does not, add it to the catalog and return a new reference.
                let attr = attribute::Attribute {
                    name: root_attr_id.clone(),
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
                };

                self.root_attributes.insert(root_attr_id, attr.clone());
                Some(self.attribute_ref(attr))
            }
        }
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
    sem_conv_catalog: &weaver_semconv::SemConvSpecs,
    version_changes: impl VersionAttributeChanges,
) -> Result<Vec<Attribute>, Error> {
    let mut resolved_attrs = BTreeMap::new();
    let mut copy_into_resolved_attrs =
        |attrs: HashMap<&String, &weaver_semconv::attribute::AttributeSpec>,
         tags: &Option<Tags>| {
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
                .attributes(attribute_group_ref, ConvTypeSpec::AttributeGroup)
                .map_err(|e| Error::FailToResolveAttributes {
                    ids: vec![attribute_group_ref.clone()],
                    error: e.to_string(),
                })?;
            copy_into_resolved_attrs(attrs, tags);
        }
    }

    // Resolve `Attribute::ResourceRef`
    for attribute in attributes.iter() {
        if let Attribute::ResourceRef { resource_ref, tags } = attribute {
            let attrs = sem_conv_catalog
                .attributes(resource_ref, ConvTypeSpec::Resource)
                .map_err(|e| Error::FailToResolveAttributes {
                    ids: vec![resource_ref.clone()],
                    error: e.to_string(),
                })?;
            copy_into_resolved_attrs(attrs, tags);
        }
    }

    // Resolve `Attribute::SpanRef`
    for attribute in attributes.iter() {
        if let Attribute::SpanRef { span_ref, tags } = attribute {
            let attrs = sem_conv_catalog
                .attributes(span_ref, ConvTypeSpec::Span)
                .map_err(|e| Error::FailToResolveAttributes {
                    ids: vec![span_ref.clone()],
                    error: e.to_string(),
                })?;
            copy_into_resolved_attrs(attrs, tags);
        }
    }

    // Resolve `Attribute::EventRef`
    for attribute in attributes.iter() {
        if let Attribute::EventRef { event_ref, tags } = attribute {
            let attrs = sem_conv_catalog
                .attributes(event_ref, ConvTypeSpec::Event)
                .map_err(|e| Error::FailToResolveAttributes {
                    ids: vec![event_ref.clone()],
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
                Error::FailToResolveAttributes {
                    ids: vec![r#ref.clone()],
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
    registry: &SemConvSpecs,
    attr: &weaver_semconv::attribute::AttributeSpec,
) -> Result<weaver_resolved_schema::attribute::Attribute, Error> {
    match attr {
        weaver_semconv::attribute::AttributeSpec::Ref { r#ref, .. } => {
            let sem_conv_attr =
                registry
                    .attribute(r#ref)
                    .ok_or(Error::FailToResolveAttributes {
                        ids: vec![r#ref.clone()],
                        error: "Attribute ref not found in the resolved registry".to_string(),
                    })?;
            resolve_attribute(registry, sem_conv_attr)
        }
        weaver_semconv::attribute::AttributeSpec::Id {
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
        } => Ok(attribute::Attribute {
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
    attr_type: &AttributeTypeSpec,
) -> weaver_resolved_schema::attribute::AttributeType {
    match attr_type {
        AttributeTypeSpec::PrimitiveOrArray(poa) => match poa {
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
        AttributeTypeSpec::Template(template) => match template {
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
        AttributeTypeSpec::Enum {
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
                        Value::Double(d) => {
                            weaver_resolved_schema::value::Value::Double { value: *d }
                        }
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
        Examples::Double(v) => attribute::Example::Double { value: *v },
        Examples::String(v) => attribute::Example::String { value: v.clone() },
        Examples::Ints(v) => weaver_resolved_schema::attribute::Example::Ints { values: v.clone() },
        Examples::Doubles(v) => attribute::Example::Doubles { values: v.clone() },
        Examples::Bools(v) => attribute::Example::Bools { values: v.clone() },
        Examples::Strings(v) => attribute::Example::Strings { values: v.clone() },
    })
}

fn semconv_to_resolved_req_level(
    req_level: &RequirementLevelSpec,
) -> weaver_resolved_schema::attribute::RequirementLevel {
    match req_level {
        RequirementLevelSpec::Basic(level) => match level {
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
        RequirementLevelSpec::Recommended { text } => {
            weaver_resolved_schema::attribute::RequirementLevel::Recommended {
                text: Some(text.clone()),
            }
        }
        RequirementLevelSpec::ConditionallyRequired { text } => {
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
        Value::Double(d) => weaver_resolved_schema::value::Value::Double { value: *d },
    })
}
