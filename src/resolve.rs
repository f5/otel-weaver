// SPDX-License-Identifier: Apache-2.0

//! Command to resolve a schema file, then output and display the results on the console.

use clap::Parser;
use std::path::PathBuf;
use std::process::exit;

use logger::Logger;
use resolver::SchemaResolver;

/// Parameters for the `resolve` command
#[derive(Parser)]
pub struct ResolveParams {
    /// Schema file to resolve
    #[arg(short, long, value_name = "FILE")]
    schema: PathBuf,

    /// Output file to write the resolved schema to
    /// If not specified, the resolved schema is printed to stdout
    #[arg(short, long)]
    output: Option<PathBuf>,
}

/// Resolve a schema file and print the result
pub fn command_resolve(log: &mut Logger, params: &ResolveParams) {
    let schema = params.schema.clone();
    let schema_name = match params.schema.to_str() {
        Some(name) => name,
        None => {
            log.error("Invalid schema name");
            exit(1)
        }
    };
    let schema = SchemaResolver::resolve_schema_file(schema, log);

    match schema {
        Ok(schema) => {
            log.success(&format!("Loaded schema {}", schema_name));
            match serde_yaml::to_string(&schema) {
                Ok(yaml) => {
                    if let Some(output) = &params.output {
                        if let Err(e) = std::fs::write(output, &yaml) {
                            log.error(&format!(
                                "Failed to write to {}: {}",
                                output.to_str().unwrap(),
                                e
                            ));
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
