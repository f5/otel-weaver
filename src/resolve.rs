// SPDX-License-Identifier: Apache-2.0

//! Command to resolve a schema file, then output and display the results on the console.

use clap::Parser;
use std::path::PathBuf;
use std::process::exit;

use weaver_logger::Logger;
use weaver_resolver::SchemaResolver;

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
pub fn command_resolve(log: &Logger, params: &ResolveParams) {
    let schema = params.schema.clone();
    let schema = SchemaResolver::resolve_schema_file(schema, log);

    match schema {
        Ok(schema) => match serde_yaml::to_string(&schema) {
            Ok(yaml) => {
                if let Some(output) = &params.output {
                    log.loading(&format!(
                        "Saving resolved schema to {}",
                        output
                            .to_str()
                            .unwrap_or("<unrepresentable-filename-not-utf8>")
                    ));
                    if let Err(e) = std::fs::write(output, &yaml) {
                        log.error(&format!(
                            "Failed to write to {}: {}",
                            output.to_str().unwrap(),
                            e
                        ));
                        exit(1)
                    }
                    log.success(&format!(
                        "Saved resolved schema to '{}'",
                        output
                            .to_str()
                            .unwrap_or("<unrepresentable-filename-not-utf8>")
                    ));
                } else {
                    log.log(&yaml);
                }
            }
            Err(e) => {
                log.error(&format!("{}", e));
                exit(1)
            }
        },
        Err(e) => {
            log.error(&format!("{}", e));
            exit(1)
        }
    }
}
