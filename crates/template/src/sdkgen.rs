// SPDX-License-Identifier: Apache-2.0

//! Client SDK generator

use crate::{Error, GeneratorConfig};

/// Client SDK generator
pub struct ClientSdkGenerator {}

impl ClientSdkGenerator {
    /// Create a new client SDK generator for the given language
    /// or return an error if the language is not supported.
    pub fn try_new(language: &str, config: GeneratorConfig) -> Result<Self, Error> {
        // Check if the language is supported
        // A language is supported if a template directory exists for it.
        if !config.template_dir.join(language).exists() {
            return Err(Error::LanguageNotSupported(language.to_string()));
        }

        Ok(Self {})
    }
}
