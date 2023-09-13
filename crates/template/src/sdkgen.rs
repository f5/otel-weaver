// SPDX-License-Identifier: Apache-2.0

//! Client SDK generator

use std::error::Error;
use std::fmt::Debug;
use std::path::PathBuf;
use std::{fs, process};

use tera::{Context, Tera};
use walkdir::WalkDir;

use logger::Logger;
use resolver::SchemaResolver;

use crate::{filters, GeneratorConfig};
use crate::Error::{InvalidTelemetrySchema, InvalidTemplate, InvalidTemplateDirectory, InvalidTemplateFile, LanguageNotSupported, WriteGeneratedCodeFailed};

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
                format!("{}/**/*.tera", dir)
            }
        };

        let mut tera = match Tera::new(&lang_dir_tree) {
            Ok(tera) => tera,
            Err(e) => {
                return Err(InvalidTemplate {
                    template: lang_path,
                    error: format!("{}", e),
                });
            }
        };
        tera.register_filter("snake_case", filters::snake_case);
        tera.register_filter("PascalCase", filters::pascal_case);
        tera.register_filter("required", filters::required);
        tera.register_filter("not_required", filters::not_required);
        tera.register_filter("convert", filters::convert);
        tera.register_filter("comment", filters::comment);
        tera.register_filter("comment_examples", filters::comment_examples);

        Ok(Self {
            lang_path,
            tera,
        })
    }

    /// Generate a client SDK for the given schema
    pub fn generate(&self,
                    log: &mut Logger,
                    schema_path: PathBuf,
                    output_dir: PathBuf,
    ) -> Result<(), crate::Error> {
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

        /// Process recursively all files in the template directory
        // let absolute_lang_path = fs::canonicalize(&self.lang_path)
        //     .map_err(|e| InvalidTemplateDirectory(self.lang_path.clone()))?;

        for entry in WalkDir::new(&self.lang_path) {
            if let Ok(entry) = entry {
                if entry.file_type().is_dir() {
                    continue;
                }
                let tmpl_file_path = entry.path();
                let relative_path = tmpl_file_path.strip_prefix(&self.lang_path).unwrap();
                let tmpl_file = relative_path.to_str()
                    .ok_or(InvalidTemplateFile(entry.path().to_path_buf()))?;

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

                // Remove the `tera` extension from the relative path
                let mut relative_path = relative_path.to_path_buf();
                relative_path.set_extension("");

                // Create all intermediary directories if they don't exist
                let output_file_path = output_dir.join(relative_path);
                if let Some(parent_dir) = output_file_path.parent() {
                    if let Err(e) = fs::create_dir_all(parent_dir) {
                        return Err(WriteGeneratedCodeFailed {
                            template: output_file_path.clone(),
                            error: format!("{}", e),
                        })
                    }
                }

                // Write the generated code to the output directory
                fs::write(output_file_path.clone(), output).map_err(|e| {
                    WriteGeneratedCodeFailed{
                        template: output_file_path.clone(),
                        error: format!("{}", e),
                    }
                })?;
                log.success(&format!("Generated file {:?}", output_file_path));
            } else {
                return Err(InvalidTemplateDirectory(self.lang_path.clone()));
            }
        }

        Ok(())
    }
}
