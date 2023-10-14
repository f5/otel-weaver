// SPDX-License-Identifier: Apache-2.0

//! Client SDK generator

use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::{fs, process};

use glob::glob;
use tera::{Context, Tera};

use logger::Logger;
use resolver::{SchemaResolver, TelemetrySchema};
use schema::univariate_metric::UnivariateMetric;

use crate::config::{DynamicGlobalConfig, LanguageConfig};
use crate::Error::{
    InternalError, InvalidTelemetrySchema, InvalidTemplate, InvalidTemplateDirectory,
    InvalidTemplateFile, LanguageNotSupported, TemplateFileNameUndefined, WriteGeneratedCodeFailed,
};
use crate::{filters, functions, testers, GeneratorConfig};

/// Client SDK generator
pub struct ClientSdkGenerator {
    /// Language path
    lang_path: PathBuf,

    /// Tera template engine
    tera: Tera,

    /// Global configuration
    config: Arc<Mutex<DynamicGlobalConfig>>,
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

        let lang_config = LanguageConfig::try_new(&lang_path)?;

        let config = Arc::new(Mutex::new(DynamicGlobalConfig::default()));

        // Register custom filters
        tera.register_filter(
            "file_name",
            filters::CaseConverter::new(lang_config.file_name, "file_name"),
        );
        tera.register_filter(
            "function_name",
            filters::CaseConverter::new(lang_config.function_name, "function_name"),
        );
        tera.register_filter(
            "arg_name",
            filters::CaseConverter::new(lang_config.arg_name, "arg_name"),
        );
        tera.register_filter(
            "struct_name",
            filters::CaseConverter::new(lang_config.struct_name, "struct_name"),
        );
        tera.register_filter(
            "field_name",
            filters::CaseConverter::new(lang_config.field_name, "field_name"),
        );
        tera.register_filter("unique_attributes", filters::unique_attributes);
        tera.register_filter("instrument", filters::instrument);
        tera.register_filter("required", filters::required);
        tera.register_filter("not_required", filters::not_required);
        tera.register_filter("with_value", filters::with_value);
        tera.register_filter("without_value", filters::without_value);
        tera.register_filter("comment", filters::comment);
        tera.register_filter(
            "type_mapping",
            filters::TypeMapping {
                type_mapping: lang_config.type_mapping,
            },
        );

        // Register custom functions
        tera.register_function("config", functions::FunctionConfig::new(config.clone()));

        // Register custom testers
        tera.register_tester("required", testers::is_required);
        tera.register_tester("not_required", testers::is_not_required);

        Ok(Self {
            lang_path,
            tera,
            config,
        })
    }

    /// Generate a client SDK for the given schema
    pub fn generate(
        &self,
        log: &Logger,
        schema_path: PathBuf,
        output_dir: PathBuf,
    ) -> Result<(), crate::Error> {
        let schema =
            SchemaResolver::resolve_schema_file(schema_path.clone(), log).map_err(|e| {
                InvalidTelemetrySchema {
                    schema: schema_path.clone(),
                    error: format!("{}", e),
                }
            })?;

        let context = &Context::from_serialize(&schema).map_err(|e| InvalidTelemetrySchema {
            schema: schema_path.clone(),
            error: format!("{}", e),
        })?;

        // Process recursively all files in the template directory
        let mut lang_path = self.lang_path.to_str().unwrap_or_default().to_string();
        let paths = if lang_path.is_empty() {
            glob("**/*.tera").map_err(|e| InternalError(e.to_string()))?
        } else {
            lang_path.push_str("/**/*.tera");
            glob(lang_path.as_str()).map_err(|e| InternalError(e.to_string()))?
        };

        for entry in paths {
            if let Ok(tmpl_file_path) = entry {
                if tmpl_file_path.is_dir() {
                    continue;
                }
                let relative_path = tmpl_file_path.strip_prefix(&self.lang_path).unwrap();
                let tmpl_file = relative_path
                    .to_str()
                    .ok_or(InvalidTemplateFile(tmpl_file_path.clone()))?;

                if tmpl_file.ends_with(".macro.tera") {
                    // Macro files are not templates.
                    // They are included in other templates.
                    // So we skip them.
                    continue;
                }

                match tmpl_file_path.file_stem().and_then(|s| s.to_str()) {
                    Some("univariate_metric") => {
                        self.process_univariate_metrics(
                            log,
                            tmpl_file,
                            &schema_path,
                            &schema,
                            &output_dir,
                        )?;
                    }
                    Some("multivariate_metric") => {
                        self.process_multivariate_metrics(
                            log,
                            tmpl_file,
                            &schema_path,
                            &schema,
                            &output_dir,
                        )?;
                    }
                    Some("log") => {
                        self.process_logs(log, tmpl_file, &schema_path, &schema, &output_dir)?;
                    }
                    Some("span") => {
                        self.process_spans(log, tmpl_file, &schema_path, &schema, &output_dir)?;
                    }
                    _ => {
                        // Process other templates
                        log.loading(&format!("Generating file {}", tmpl_file));
                        let generated_code = self.generate_code(log, tmpl_file, context)?;

                        // Remove the `tera` extension from the relative path
                        let mut relative_path = relative_path.to_path_buf();
                        relative_path.set_extension("");

                        let generated_file =
                            Self::save_generated_code(&output_dir, relative_path, generated_code)?;
                        log.success(&format!("Generated file {:?}", generated_file));
                    }
                }
            } else {
                return Err(InvalidTemplateDirectory(self.lang_path.clone()));
            }
        }

        Ok(())
    }

    /// Generate code.
    fn generate_code(
        &self,
        log: &Logger,
        tmpl_file: &str,
        context: &Context,
    ) -> Result<String, crate::Error> {
        let generated_code = self.tera.render(tmpl_file, context).unwrap_or_else(|err| {
            log.newline(1);
            log.error(&format!("{}", err));
            let mut cause = err.source();
            while let Some(e) = cause {
                log.error(&format!("Caused by: {}", e));
                cause = e.source();
            }
            process::exit(1);
        });

        Ok(generated_code)
    }

    /// Save the generated code to the output directory.
    fn save_generated_code(
        output_dir: &Path,
        relative_path: PathBuf,
        generated_code: String,
    ) -> Result<PathBuf, crate::Error> {
        // Create all intermediary directories if they don't exist
        let output_file_path = output_dir.join(relative_path);
        if let Some(parent_dir) = output_file_path.parent() {
            if let Err(e) = fs::create_dir_all(parent_dir) {
                return Err(WriteGeneratedCodeFailed {
                    template: output_file_path.clone(),
                    error: format!("{}", e),
                });
            }
        }

        // Write the generated code to the output directory
        fs::write(output_file_path.clone(), generated_code).map_err(|e| {
            WriteGeneratedCodeFailed {
                template: output_file_path.clone(),
                error: format!("{}", e),
            }
        })?;

        Ok(output_file_path)
    }

    /// Process all univariate metrics in the schema.
    fn process_univariate_metrics(
        &self,
        log: &Logger,
        tmpl_file: &str,
        schema_path: &Path,
        schema: &TelemetrySchema,
        output_dir: &Path,
    ) -> Result<(), crate::Error> {
        if let Some(schema_spec) = &schema.schema {
            if let Some(metrics) = schema_spec.resource_metrics.as_ref() {
                for metric in metrics.metrics.iter() {
                    if let UnivariateMetric::Metric { name, .. } = metric {
                        let context = &Context::from_serialize(metric).map_err(|e| {
                            InvalidTelemetrySchema {
                                schema: schema_path.to_path_buf(),
                                error: format!("{}", e),
                            }
                        })?;

                        // Reset the config
                        {
                            self.config
                                .lock()
                                .map_err(|e| InternalError(e.to_string()))?
                                .reset();
                        }

                        log.loading(&format!("Generating code for univariate metric `{}`", name));
                        let generated_code = self.generate_code(log, tmpl_file, context)?;

                        // Retrieve the file name from the config
                        let relative_path = {
                            let mutex_guard = self
                                .config
                                .lock()
                                .map_err(|e| InternalError(e.to_string()))?;
                            match &mutex_guard.file_name {
                                None => {
                                    return Err(TemplateFileNameUndefined {
                                        template: PathBuf::from(tmpl_file),
                                    });
                                }
                                Some(file_name) => PathBuf::from(file_name.clone()),
                            }
                        };

                        // Save the generated code to the output directory
                        let generated_file =
                            Self::save_generated_code(output_dir, relative_path, generated_code)?;
                        log.success(&format!("Generated file {:?}", generated_file));
                    }
                }
            }
        }
        Ok(())
    }

    /// Process all multivariate metrics in the schema.
    fn process_multivariate_metrics(
        &self,
        log: &Logger,
        tmpl_file: &str,
        schema_path: &Path,
        schema: &TelemetrySchema,
        output_dir: &Path,
    ) -> Result<(), crate::Error> {
        if let Some(schema_spec) = &schema.schema {
            if let Some(metrics) = schema_spec.resource_metrics.as_ref() {
                for metric in metrics.metric_groups.iter() {
                    let context =
                        &Context::from_serialize(metric).map_err(|e| InvalidTelemetrySchema {
                            schema: schema_path.to_path_buf(),
                            error: format!("{}", e),
                        })?;

                    // Reset the config
                    {
                        self.config
                            .lock()
                            .map_err(|e| InternalError(e.to_string()))?
                            .reset();
                    }

                    log.loading(&format!(
                        "Generating code for multivariate metric `{}`",
                        metric.id
                    ));
                    let generated_code = self.generate_code(log, tmpl_file, context)?;

                    // Retrieve the file name from the config
                    let relative_path = {
                        let mutex_guard = self
                            .config
                            .lock()
                            .map_err(|e| InternalError(e.to_string()))?;
                        match &mutex_guard.file_name {
                            None => {
                                return Err(TemplateFileNameUndefined {
                                    template: PathBuf::from(tmpl_file),
                                });
                            }
                            Some(file_name) => PathBuf::from(file_name.clone()),
                        }
                    };

                    // Save the generated code to the output directory
                    let generated_file =
                        Self::save_generated_code(output_dir, relative_path, generated_code)?;
                    log.success(&format!("Generated file {:?}", generated_file));
                }
            }
        }
        Ok(())
    }

    /// Process all logs in the schema.
    fn process_logs(
        &self,
        log: &Logger,
        tmpl_file: &str,
        schema_path: &Path,
        schema: &TelemetrySchema,
        output_dir: &Path,
    ) -> Result<(), crate::Error> {
        if let Some(schema_spec) = &schema.schema {
            if let Some(logs) = schema_spec.resource_events.as_ref() {
                for log_record in logs.events.iter() {
                    let context = &Context::from_serialize(log_record).map_err(|e| {
                        InvalidTelemetrySchema {
                            schema: schema_path.to_path_buf(),
                            error: format!("{}", e),
                        }
                    })?;

                    // Reset the config
                    {
                        self.config
                            .lock()
                            .map_err(|e| InternalError(e.to_string()))?
                            .reset();
                    }

                    log.loading(&format!(
                        "Generating code for log `{}`",
                        log_record.event_name
                    ));
                    let generated_code = self.generate_code(log, tmpl_file, context)?;

                    // Retrieve the file name from the config
                    let relative_path = {
                        let mutex_guard = self
                            .config
                            .lock()
                            .map_err(|e| InternalError(e.to_string()))?;
                        match &mutex_guard.file_name {
                            None => {
                                return Err(TemplateFileNameUndefined {
                                    template: PathBuf::from(tmpl_file),
                                });
                            }
                            Some(file_name) => PathBuf::from(file_name.clone()),
                        }
                    };

                    // Save the generated code to the output directory
                    let generated_file =
                        Self::save_generated_code(output_dir, relative_path, generated_code)?;
                    log.success(&format!("Generated file {:?}", generated_file));
                }
            }
        }
        Ok(())
    }

    /// Process all spans in the schema.
    fn process_spans(
        &self,
        log: &Logger,
        tmpl_file: &str,
        schema_path: &Path,
        schema: &TelemetrySchema,
        output_dir: &Path,
    ) -> Result<(), crate::Error> {
        if let Some(schema_spec) = &schema.schema {
            if let Some(spans) = schema_spec.resource_spans.as_ref() {
                for span in spans.spans.iter() {
                    let context =
                        &Context::from_serialize(span).map_err(|e| InvalidTelemetrySchema {
                            schema: schema_path.to_path_buf(),
                            error: format!("{}", e),
                        })?;

                    // Reset the config
                    {
                        self.config
                            .lock()
                            .map_err(|e| InternalError(e.to_string()))?
                            .reset();
                    }

                    log.loading(&format!("Generating code for span `{}`", span.span_name));
                    let generated_code = self.generate_code(log, tmpl_file, context)?;

                    // Retrieve the file name from the config
                    let relative_path = {
                        let mutex_guard = self
                            .config
                            .lock()
                            .map_err(|e| InternalError(e.to_string()))?;
                        match &mutex_guard.file_name {
                            None => {
                                return Err(TemplateFileNameUndefined {
                                    template: PathBuf::from(tmpl_file),
                                });
                            }
                            Some(file_name) => PathBuf::from(file_name.clone()),
                        }
                    };

                    // Save the generated code to the output directory
                    let generated_file =
                        Self::save_generated_code(output_dir, relative_path, generated_code)?;
                    log.success(&format!("Generated file {:?}", generated_file));
                }
            }
        }
        Ok(())
    }
}
