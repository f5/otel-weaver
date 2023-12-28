// SPDX-License-Identifier: Apache-2.0

//! Functions to resolve a semantic convention registry.

use weaver_logger::Logger;
use weaver_resolved_schema::attribute::{AttributeRef, UnresolvedAttribute};
use weaver_resolved_schema::registry::{
    Group, Registry, TypedGroup, UnresolvedGroup, UnresolvedRegistry,
};
use weaver_semconv::group::{ConvTypeSpec, GroupSpec};
use weaver_semconv::SemConvSpecs;

use crate::attribute::{resolve_attribute, AttributeCatalog};
use crate::constraint::resolve_constraints;
use crate::metrics::resolve_instrument;
use crate::spans::resolve_span_kind;
use crate::stability::resolve_stability;
use crate::Error;

/// Creates a registry from a set of semantic convention specifications.
/// Note: this function does not resolve references.
#[allow(dead_code)] // ToDo remove this once this function is called from the CLI.
pub fn unresolved_registry_from_specs(url: &str, specs: &SemConvSpecs) -> UnresolvedRegistry {
    let groups = specs.groups().map(group_from_spec).collect();

    UnresolvedRegistry {
        registry: Registry {
            registry_url: url.to_string(),
            groups: vec![],
        },
        groups,
    }
}

/// Creates a group from a semantic convention group specification.
/// Note: this function does not resolve references.
fn group_from_spec(group: &GroupSpec) -> UnresolvedGroup {
    let attrs = group
        .attributes
        .iter()
        .map(|attr| UnresolvedAttribute { spec: attr.clone() })
        .collect();

    UnresolvedGroup {
        group: Group {
            id: group.id.clone(),
            typed_group: match group.r#type {
                ConvTypeSpec::AttributeGroup => TypedGroup::AttributeGroup {},
                ConvTypeSpec::Span => TypedGroup::Span {
                    span_kind: group.span_kind.as_ref().map(resolve_span_kind),
                    events: group.events.clone(),
                },
                ConvTypeSpec::Event => TypedGroup::Event {
                    name: group.name.clone(),
                },
                ConvTypeSpec::Metric => TypedGroup::Metric {
                    metric_name: group.metric_name.clone(),
                    instrument: group.instrument.as_ref().map(resolve_instrument),
                    unit: group.unit.clone(),
                },
                ConvTypeSpec::MetricGroup => TypedGroup::MetricGroup {},
                ConvTypeSpec::Resource => TypedGroup::Resource {},
                ConvTypeSpec::Scope => TypedGroup::Scope {},
            },
            brief: group.brief.to_string(),
            note: group.note.to_string(),
            prefix: group.prefix.to_string(),
            extends: group.extends.clone(),
            stability: resolve_stability(&group.stability),
            deprecated: group.deprecated.clone(),
            constraints: resolve_constraints(&group.constraints),
            attributes: vec![],
        },
        attributes: attrs,
    }
}

/// Resolve a semantic convention registry.
pub fn resolve_semconv_registry(
    attr_catalog: &mut AttributeCatalog,
    url: &str,
    registry: &SemConvSpecs,
    _log: impl Logger + Sync + Clone,
) -> Result<Registry, Error> {
    let groups: Result<Vec<weaver_resolved_schema::registry::Group>, Error> = registry
        .groups()
        .map(|group| semconv_to_resolved_group(registry, attr_catalog, group))
        .collect();

    Ok(Registry {
        registry_url: url.to_string(),
        groups: groups?,
    })
}

/// Resolve a semantic convention group.
fn semconv_to_resolved_group(
    registry: &SemConvSpecs,
    attr_catalog: &mut AttributeCatalog,
    group: &GroupSpec,
) -> Result<weaver_resolved_schema::registry::Group, Error> {
    let attr_refs: Result<Vec<AttributeRef>, Error> = group
        .attributes
        .iter()
        .map(|attr| Ok(attr_catalog.attribute_ref(resolve_attribute(registry, attr)?)))
        .collect();

    Ok(weaver_resolved_schema::registry::Group {
        id: group.id.clone(),
        typed_group: match group.r#type {
            ConvTypeSpec::AttributeGroup => {
                weaver_resolved_schema::registry::TypedGroup::AttributeGroup {}
            }
            ConvTypeSpec::Span => weaver_resolved_schema::registry::TypedGroup::Span {
                span_kind: group.span_kind.as_ref().map(resolve_span_kind),
                events: group.events.clone(),
            },
            ConvTypeSpec::Event => weaver_resolved_schema::registry::TypedGroup::Event {
                name: group.name.clone(),
            },
            ConvTypeSpec::Metric => weaver_resolved_schema::registry::TypedGroup::Metric {
                metric_name: group.metric_name.clone(),
                instrument: group.instrument.as_ref().map(resolve_instrument),
                unit: group.unit.clone(),
            },
            ConvTypeSpec::MetricGroup => {
                weaver_resolved_schema::registry::TypedGroup::MetricGroup {}
            }
            ConvTypeSpec::Resource => weaver_resolved_schema::registry::TypedGroup::Resource {},
            ConvTypeSpec::Scope => weaver_resolved_schema::registry::TypedGroup::Scope {},
        },
        brief: group.brief.to_string(),
        note: group.note.to_string(),
        prefix: group.prefix.to_string(),
        extends: group.extends.clone(),
        stability: resolve_stability(&group.stability),
        deprecated: group.deprecated.clone(),
        constraints: resolve_constraints(&group.constraints),
        attributes: attr_refs?,
    })
}

/// Resolves the registry by resolving all groups and attributes.
#[allow(dead_code)] // ToDo remove this once this function is called from the CLI.
pub fn resolve_registry(
    mut ureg: UnresolvedRegistry,
    attr_catalog: &mut AttributeCatalog,
) -> Result<Registry, Error> {
    loop {
        let mut unresolved_attr_count = 0;
        let mut resolved_attr_count = 0;

        // Iterate over all groups and resolve the attributes.
        for unresolved_group in ureg.groups.iter_mut() {
            let mut resolved_attr = vec![];

            unresolved_group.attributes = unresolved_group
                .attributes
                .clone()
                .into_iter()
                .filter_map(|attr| {
                    let attr_ref = attr_catalog.resolve(&unresolved_group.group.prefix, &attr.spec);
                    if let Some(attr_ref) = attr_ref {
                        resolved_attr.push(attr_ref);
                        resolved_attr_count += 1;
                        None
                    } else {
                        unresolved_attr_count += 1;
                        Some(attr)
                    }
                })
                .collect();

            unresolved_group.group.attributes.extend(resolved_attr);
        }

        if unresolved_attr_count == 0 {
            break;
        }
        // If we still have unresolved attributes but we did not resolve any
        // attributes in the last iteration, we are stuck in an infinite loop.
        // It means that we have an issue with the semantic convention
        // specifications.
        if resolved_attr_count == 0 {
            return Err(Error::FailToResolveAttributes {
                ids: ureg
                    .groups
                    .iter()
                    .flat_map(|g| g.attributes.iter().map(|attr| attr.spec.id()))
                    .collect(),
                error: "".to_string(),
            });
        }
    }

    ureg.registry.groups = ureg.groups.into_iter().map(|g| g.group).collect();
    Ok(ureg.registry)
}

#[cfg(test)]
mod tests {
    use glob::glob;

    use weaver_resolved_schema::attribute;
    use weaver_resolved_schema::registry::Registry;
    use weaver_semconv::SemConvSpecs;

    use crate::attribute::AttributeCatalog;
    use crate::registry::{resolve_registry, unresolved_registry_from_specs};

    /// Test the resolution of semantic convention registries stored in the
    /// data directory.
    ///
    /// Each test is stored in a directory named `registry-test-*` and contains
    /// the following directory and files:
    /// - directory `registry` containing the semantic convention specifications
    ///   in YAML format.
    /// - file `expected-attribute-catalog.json` containing the expected
    ///   attribute catalog in JSON format.
    /// - file `expected-registry.json` containing the expected registry in
    ///   JSON format.
    #[test]
    fn test_registry_resolution() {
        // Iterate over all directories in the data directory and
        // starting with registry-test-*
        for test_entry in glob("data/registry-test-*").expect("Failed to read glob pattern") {
            let path_buf = test_entry.expect("Failed to read test directory");
            let test_dir = path_buf
                .to_str()
                .expect("Failed to convert test directory to string");

            println!("Testing `{}`", test_dir);

            let mut sc_specs = SemConvSpecs::default();
            for sc_entry in
                glob(&format!("{}/registry/*.yaml", test_dir)).expect("Failed to read glob pattern")
            {
                let path_buf = sc_entry.expect("Failed to read semconv file");
                let semconv_file = path_buf
                    .to_str()
                    .expect("Failed to convert semconv file to string");
                let result = sc_specs.load_from_file(semconv_file);
                assert!(
                    result.is_ok(),
                    "Failed to load semconv file `{}, error: {:#?}",
                    semconv_file,
                    result.err().unwrap()
                );
            }

            let mut attr_catalog = AttributeCatalog::default();
            let observed_registry = resolve_registry(
                unresolved_registry_from_specs("https://semconv-registry.com", &sc_specs),
                &mut attr_catalog,
            )
            .expect("Failed to resolve registry");

            // Load the expected registry and attribute catalog.
            let expected_attr_catalog: Vec<attribute::Attribute> = serde_json::from_reader(
                std::fs::File::open(format!("{}/expected-attribute-catalog.json", test_dir))
                    .expect("Failed to open expected attribute catalog"),
            )
            .expect("Failed to deserialize expected attribute catalog");
            let expected_registry: Registry = serde_json::from_reader(
                std::fs::File::open(format!("{}/expected-registry.json", test_dir))
                    .expect("Failed to open expected registry"),
            )
            .expect("Failed to deserialize expected registry");

            // Check that the resolved attribute catalog matches the expected attribute catalog.
            let observed_attr_catalog = attr_catalog.drain_attributes();
            let observed_attr_catalog_json = serde_json::to_string_pretty(&observed_attr_catalog)
                .expect("Failed to serialize observed attribute catalog");

            assert_eq!(
                observed_attr_catalog, expected_attr_catalog,
                "Attribute catalog does not match for `{}`.\nObserved catalog:\n{}",
                test_dir, observed_attr_catalog_json
            );

            // Check that the resolved registry matches the expected registry.
            let observed_registry_json = serde_json::to_string_pretty(&observed_registry)
                .expect("Failed to serialize observed registry");

            assert_eq!(
                observed_registry, expected_registry,
                "Registry does not match for `{}`.\nObserved registry:\n{}",
                test_dir, observed_registry_json
            );
        }
    }
}

// ToDo Remove #[allow(dead_code)] once the corresponding functions are called from the CLI.
// ToDo Work on the metrics, spans, events, ... resolutions.
