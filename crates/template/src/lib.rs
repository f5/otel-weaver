use std::path::PathBuf;

pub mod sdkgen;

/// An error that can occur while generating a client SDK.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Language not found.
    #[error("Language `{0}` is not supported. Use the command `languages` to list supported languages.")]
    LanguageNotSupported(String),
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