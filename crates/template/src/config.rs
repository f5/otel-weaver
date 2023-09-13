// SPDX-License-Identifier: Apache-2.0

//! Configuration for the template crate.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;

use crate::Error;
use crate::Error::InvalidConfigFile;

#[derive(Deserialize, Debug, Default)]
pub struct LanguageConfig {
    #[serde(default)]
    pub type_mapping: HashMap<String, String>,
}

#[derive(Debug, Default)]
pub struct DynamicGlobalConfig {
    pub file_name: Option<String>,
}

impl LanguageConfig {
    pub fn try_new(lang_path: &PathBuf) -> Result<LanguageConfig, Error> {
        let config_file = lang_path.join("config.yaml");
        if config_file.exists() {
            let reader = std::fs::File::open(config_file.clone()).map_err(|e|
                InvalidConfigFile {
                    config_file: config_file.clone(),
                    error: e.to_string(),
                }
            )?;
            serde_yaml::from_reader(reader).map_err(|e|
                InvalidConfigFile {
                    config_file: config_file.clone(),
                    error: e.to_string(),
                }
            )
        } else {
            Ok(LanguageConfig::default())
        }
    }
}

impl DynamicGlobalConfig {
    pub fn reset(&mut self) {
        self.file_name = None;
    }
}
