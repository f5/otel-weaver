// SPDX-License-Identifier: Apache-2.0

//! Custom Tera functions

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, LockResult, Mutex};
use tera::{Function, Value};
use tera::Result;
use crate::config::DynamicGlobalConfig;


#[derive(Debug)]
pub struct FunctionConfig {
    config: Arc<Mutex<DynamicGlobalConfig>>,
}

impl FunctionConfig {
    pub fn new(config: Arc<Mutex<DynamicGlobalConfig>>) -> Self {
        FunctionConfig {
            config,
        }
    }
}

impl Function for FunctionConfig {
    fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
        if let Some(file_name) = args.get("file_name") {
            if let Ok(mut config) = self.config.lock() {
                // update file_name
                config.file_name = Some(file_name.as_str().unwrap().to_string());
            }
        }
        Ok(Value::Null)
    }

    fn is_safe(&self) -> bool {
        false
    }
}