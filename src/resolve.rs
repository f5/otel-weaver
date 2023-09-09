// SPDX-License-Identifier: Apache-2.0

//! Resolve a schema file and print the result

use std::path::PathBuf;
use std::process::exit;
use clap::Parser;

use logger::Logger;
use resolver::SchemaResolver;

#[derive(Parser)]
pub struct Resolve {
    /// Schema file to resolve
    #[arg(short, long, value_name = "FILE")]
    schema: PathBuf,

    /// Output file to write the resolved schema to
    /// If not specified, the resolved schema is printed to stdout
    #[arg(short, long)]
    output: Option<PathBuf>,
}

/// Resolve a schema file and print the result
pub fn command_resolve(log: &mut Logger, params: &Resolve) {
    let schema = params.schema.clone();
    let schema_name = params.schema.to_str().expect("Invalid schema name");
    let schema = SchemaResolver::resolve_schema_file(schema, log);

    match schema {
        Ok(schema) => {
            log.success(&format!("Loaded schema {}", schema_name));
            match serde_yaml::to_string(&schema) {
                Ok(yaml) => {
                    if let Some(output) = &params.output {
                        if let Err(e) = std::fs::write(output, &yaml) {
                            log.error(&format!("Failed to write to {}: {}", output.to_str().unwrap(), e));
                            exit(1)
                        }
                    } else {
                        log.log(&yaml);
                    }
                }
                Err(e) => {
                    log.error(&format!("{}", e));
                    exit(1)
                }
            }
        }
        Err(e) => {
            log.error(&format!("{}", e));
            exit(1)
        }
    }
}
