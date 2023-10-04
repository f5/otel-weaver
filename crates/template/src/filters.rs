// SPDX-License-Identifier: Apache-2.0

//! Custom Tera filters

use std::borrow::Cow;
use std::clone;
use std::collections::HashMap;

use tera::{Filter, Result, try_get_value, Value};
use textwrap::{Options, wrap};

use crate::config::CaseConvention;

/// Case converter filter.
pub struct CaseConverter {
    filter_name: &'static str,
    case: CaseConvention,
}

impl CaseConverter {
    /// Create a new case converter filter.
    pub fn new(case: CaseConvention, filter_name: &'static str) -> Self {
        CaseConverter {
            filter_name,
            case,
        }
    }
}

/// Filter to convert a string to a specific case.
impl Filter for CaseConverter {
    /// Convert a string to a specific case.
    fn filter(&self, value: &Value, args: &HashMap<String, Value>) -> Result<Value> {
        let text = try_get_value!(self.filter_name, "value", String, value);
        Ok(Value::String(self.case.convert(&text)))
    }
}

/// Filter to normalize instrument name.
pub fn instrument(value: &Value, _: &HashMap<String, Value>) -> Result<Value> {
    if let Value::String(metric_type) = value {
        match metric_type.as_str() {
            "counter" | "gauge" | "histogram" => return Ok(Value::String(metric_type.clone())),
            "updowncounter" => return Ok(Value::String("up_down_counter".to_string())),
            _ => return Err(tera::Error::msg(format!("Filter instrument: unknown metric instrument {}", metric_type)))
        }
    } else {
        return Err(tera::Error::msg(format!("Filter instrument: expected a string, got {:?}", value)));
    }
}

/// Filter out attributes that are not required.
pub fn required(value: &Value, _: &HashMap<String, Value>) -> Result<Value> {
    let mut required_values = vec![];
    match value {
        Value::Array(values) => {
            for value in values {
                match value {
                    Value::Object(map) => {
                        if let Some(Value::String(req_level)) = map.get("requirement_level") {
                            if req_level == "required" {
                                required_values.push(value.clone());
                            }
                        }
                    }
                    _ => required_values.push(value.clone())
                }
            }
        }
        _ => return Ok(value.clone())
    }
    Ok(Value::Array(required_values))
}

/// Filter out attributes that are required.
pub fn not_required(value: &Value, _: &HashMap<String, Value>) -> Result<Value> {
    let mut required_values = vec![];
    match value {
        Value::Array(values) => {
            for value in values {
                match value {
                    Value::Object(map) => {
                        if let Some(Value::String(req_level)) = map.get("requirement_level") {
                            if req_level != "required" {
                                required_values.push(value.clone());
                            }
                        } else {
                            required_values.push(value.clone());
                        }
                    }
                    _ => required_values.push(value.clone())
                }
            }
        }
        _ => return Ok(value.clone())
    }
    Ok(Value::Array(required_values))
}

/// Filter to map an OTel type to a language type.
pub struct TypeMapping {
    pub type_mapping: HashMap<String, String>,
}

impl Filter for TypeMapping {
    /// Map an OTel type to a language type.
    fn filter(&self, value: &Value, args: &HashMap<String, Value>) -> Result<Value> {
        let otel_type = try_get_value!("type_mapping", "value", String, value);

        match self.type_mapping.get(&otel_type) {
            Some(language_type) => Ok(Value::String(language_type.clone())),
            None => Err(tera::Error::msg(format!("Filter type_mapping: could not find a conversion for {}. To resolve this, create or extend the type_mapping in the config.yaml file.", otel_type)))
        }
    }
}

/// Creates a multiline comment from a string.
/// The `value` parameter is a string.
/// The `prefix` parameter is a string.
pub fn comment(value: &Value, ctx: &HashMap<String, Value>) -> Result<Value> {
    fn wrap_comment(comment: &str, prefix: &str, lines: &mut Vec<String>) {
        wrap(comment.trim_end(), Options::new(80))
            .into_iter()
            .map(|s| format!("{}{}",prefix, s.trim_end()))
            .for_each(|s| lines.push(s));
    }

    let prefix = match ctx.get("prefix") {
        Some(Value::String(prefix)) => prefix.clone(),
        _ => { "".to_string() }
    };

    let mut lines = vec![];
    match value {
        Value::String(value) => wrap_comment(value, "", &mut lines),
        Value::Array(values) => {
            for value in values {
                match value {
                    Value::String(value) => wrap_comment(value, "", &mut lines),
                    Value::Array(values) => {
                        for value in values {
                            match value {
                                Value::String(value) => wrap_comment(value, "- ", &mut lines),
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }

    let mut comments = String::new();
    for (i, line) in lines.into_iter().enumerate() {
        if i >0 {
            comments.push_str(format!("\n{}", prefix).as_ref());
        }
        comments.push_str(line.as_ref());
    }
    Ok(Value::String(comments))
}



fn uppercase_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}