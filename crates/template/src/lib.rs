use std::path::PathBuf;

pub mod sdkgen;

/// An error that can occur while generating a client SDK.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Language not found.
    #[error("Language `{0}` is not supported. Use the command `languages` to list supported languages.")]
    LanguageNotSupported(String),

    /// Invalid template directory.
    #[error("Invalid template directory: {0}")]
    InvalidTemplateDirectory(PathBuf),

    /// Invalid template file.
    #[error("Invalid template file: {0}")]
    InvalidTemplateFile(PathBuf),

    /// Invalid template.
    #[error("{error}")]
    InvalidTemplate {
        /// Template directory.
        template: PathBuf,
        /// Error message.
        error: String,
    },

    /// Invalid telemetry schema.
    #[error("Invalid telemetry schema {schema}: {error}")]
    InvalidTelemetrySchema {
        /// Schema file.
        schema: PathBuf,
        /// Error message.
        error: String,
    },
}

/// General configuration for the generator.
pub struct GeneratorConfig {
    template_dir: PathBuf,
}

impl Default for GeneratorConfig {
    /// Create a new generator configuration with default values.
    fn default() -> Self {
        Self {
            template_dir: PathBuf::from("templates"),
        }
    }
}