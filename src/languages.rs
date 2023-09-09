// SPDX-License-Identifier: Apache-2.0

//! List of supported languages

use std::path::PathBuf;
use std::process::exit;
use clap::Parser;

use logger::Logger;
use resolver::SchemaResolver;
use crate::resolve::ResolveParams;

/// Parameters for the `languages` command
#[derive(Parser)]
pub struct LanguagesParams {
    /// Template root directory
    #[arg(short, long, default_value = "templates")]
    templates: PathBuf,
}

/// List of supported languages
pub fn command_languages(log: &mut Logger, params: &LanguagesParams) {
    /// List all directories in the templates directory
    log.log("List of supported languages:");
    for entry in std::fs::read_dir(&params.templates).expect("Failed to read templates directory") {
        let entry = entry.expect("Failed to read template directory entry");
        if entry.file_type().expect("Failed to read file type").is_dir() {
            log.indent(1);
            log.log(&format!("- {}", entry.file_name().to_str().unwrap()));
        }
    }
}
