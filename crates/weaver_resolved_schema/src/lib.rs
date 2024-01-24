// SPDX-License-Identifier: Apache-2.0

//! Define the concept of Resolved Telemetry Schema.
//! A Resolved Telemetry Schema is self-contained and doesn't contain any
//! external references to other schemas or semantic conventions.

#![deny(missing_docs)]
#![deny(clippy::print_stdout)]
#![deny(clippy::print_stderr)]

use crate::catalog::Catalog;
use crate::instrumentation_library::InstrumentationLibrary;
use crate::registry::Registry;
use crate::resource::Resource;
use serde::{Deserialize, Serialize};
use weaver_version::Versions;

pub mod attribute;
pub mod catalog;
pub mod instrumentation_library;
pub mod lineage;
pub mod metric;
pub mod registry;
pub mod resource;
pub mod signal;
pub mod tags;
pub mod value;

/// Weaver protobuf definitions.
pub mod weaver {
    /// Protobuf definitions for the Weaver resolved schema.
    pub mod resolved_schema {
        include!(concat!(env!("OUT_DIR"), "/weaver.resolved_schema.rs"));
    }
}

/// A Resolved Telemetry Schema.
/// A Resolved Telemetry Schema is self-contained and doesn't contain any
/// external references to other schemas or semantic conventions.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ResolvedTelemetrySchema {
    /// Version of the file structure.
    pub file_format: String,
    /// Schema URL that this file is published at.
    pub schema_url: String,
    /// A list of semantic convention registries that can be used in this schema
    /// and its descendants.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub registries: Vec<Registry>,
    /// Catalog of unique items that are shared across multiple registries
    /// and signals.
    pub catalog: Catalog,
    /// Resource definition (only for application).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<Resource>,
    /// Definition of the instrumentation library for the instrumented application or library.
    /// Or none if the resolved telemetry schema represents a semantic convention registry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instrumentation_library: Option<InstrumentationLibrary>,
    /// The list of dependencies of the current instrumentation application or library.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<InstrumentationLibrary>,
    /// Definitions for each schema version in this family.
    /// Note: the ordering of versions is defined according to semver
    /// version number ordering rules.
    /// This section is described in more details in the OTEP 0152 and in a dedicated
    /// section below.
    /// <https://github.com/open-telemetry/oteps/blob/main/text/0152-telemetry-schemas.md>
    #[serde(skip_serializing_if = "Option::is_none")]
    pub versions: Option<Versions>,
}

#[cfg(test)]
mod test {
    use crate::weaver::resolved_schema::attribute_type::Type;
    use crate::weaver::resolved_schema::{
        example, AttributeType, BooleanAttribute, Catalog, Example, Instrument, Metric,
        ResolvedTelemetrySchema,
    };

    #[test]
    fn test() {
        let schema = ResolvedTelemetrySchema {
            file_format: "weaver".to_string(),
            schema_url: "https://schema.weaver.org".to_string(),
            registries: vec![],
            catalog: Catalog {
                attributes: vec![crate::weaver::resolved_schema::Attribute {
                    name: "".to_string(),
                    r#type: AttributeType {
                        r#type: Some(Type::Boolean(BooleanAttribute {})),
                    },
                    brief: "".to_string(),
                    examples: vec![Example {
                        r#type: Some(example::Type::Double(0.0)),
                    }],
                    tag: None,
                    requirement_level: None,
                    sampling_relevant: false,
                    note: None,
                    stability: None,
                    deprecated: None,
                    tags: Default::default(),
                    value: None,
                }],
                metrics: vec![Metric {
                    name: "metric1".to_string(),
                    brief: "brief blabla".to_string(),
                    note: "note blabla".to_string(),
                    instrument: Instrument::Counter as i32,
                    unit: Some("1".to_string()),
                    tags: Default::default(),
                }],
            },
            resource: None,
            instrumentation_library: None,
            dependencies: vec![],
        };

        let json = serde_json::to_string_pretty(&schema).unwrap();
        println!("{}", json);
    }
}
