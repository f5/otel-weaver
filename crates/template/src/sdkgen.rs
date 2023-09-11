// SPDX-License-Identifier: Apache-2.0

//! Client SDK generator

use std::error::Error;
use std::fmt::Debug;
use std::path::PathBuf;
use std::process;

use tera::{Context, Tera};

use logger::Logger;
use resolver::SchemaResolver;

use crate::{GeneratorConfig};
use crate::Error::{InvalidTelemetrySchema, InvalidTemplate, InvalidTemplateDirectory, InvalidTemplateFile, LanguageNotSupported};

/// Client SDK generator
pub struct ClientSdkGenerator {
    /// Language path
    lang_path: PathBuf,

    /// Tera template engine
    tera: Tera,
}

impl ClientSdkGenerator {
    /// Create a new client SDK generator for the given language
    /// or return an error if the language is not supported.
    pub fn try_new(language: &str, config: GeneratorConfig) -> Result<Self, crate::Error> {
        // Check if the language is supported
        // A language is supported if a template directory exists for it.
        let lang_path = config.template_dir.join(language);

        if !lang_path.exists() {
            return Err(LanguageNotSupported(language.to_string()));
        }

        let lang_dir_tree = match lang_path.to_str() {
            None => {
                return Err(InvalidTemplateDirectory(lang_path));
            }
            Some(dir) => {
                format!("{}/**/*", dir)
            }
        };

        let tera = match Tera::new(&lang_dir_tree) {
            Ok(tera) => tera,
            Err(e) => {
                return Err(InvalidTemplate {
                    template: lang_path,
                    error: format!("{}", e),
                });
            }
        };

        Ok(Self {
            lang_path,
            tera,
        })
    }

    /// Generate a client SDK for the given schema
    pub fn generate(&self, log: &mut Logger, schema_path: PathBuf) -> Result<(), crate::Error> {
        let schema = SchemaResolver::resolve_schema_file(schema_path.clone(), log).map_err(|e| {
            InvalidTelemetrySchema {
                schema: schema_path.clone(),
                error: format!("{}", e),
            }
        })?;

        let context = &Context::from_serialize(&schema).map_err(|e| {
            InvalidTelemetrySchema {
                schema: schema_path.clone(),
                error: format!("{}", e),
            }
        })?;

        /// List recursively all files in the template directory
        let lang_dir = match std::fs::read_dir(&self.lang_path) {
            Ok(dir) => dir,
            Err(e) => {
                return Err(InvalidTemplateDirectory(self.lang_path.clone()));
            }
        };

        for entry in lang_dir {
            if let Ok(entry) = entry {
                if entry.file_type().is_ok() {
                    let tmpl_file_path = entry.path();
                    let tmpl_file = tmpl_file_path.file_name()
                        .ok_or(InvalidTemplateFile(entry.path()))?
                        .to_str().ok_or(InvalidTemplateFile(entry.path()))?;

                    log.loading(&format!("Generating file {}", tmpl_file));
                    let output = self.tera.render(tmpl_file, &context).unwrap_or_else(|err| {
                        log.newline(1);
                        log.error(&format!("{}", err));
                        let mut cause = err.source();
                        while let Some(e) = cause {
                            log.error(&format!("Caused by: {}", e));
                            cause = e.source();
                        }
                        process::exit(1);
                    });
                    log.success(&format!("Generated file {:?}", tmpl_file));
                    println!("Result: {}", output);
                }
            } else {
                return Err(InvalidTemplateDirectory(self.lang_path.clone()));
            }
        }

        Ok(())
    }
}
