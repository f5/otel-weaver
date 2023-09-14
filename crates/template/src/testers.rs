// SPDX-License-Identifier: Apache-2.0

//! Custom testers

use tera::Value;

pub fn is_required(value: Option<&Value>, _args: &[Value]) -> tera::Result<bool> {
    match value {
        Some(Value::Object(map)) => {
            if let Some(Value::String(req_level)) = map.get("requirement_level") {
                if req_level == "required" {
                    return Ok(true)
                }
            }
        }
        _ => {}
    }
    return Ok(false)
}

pub fn is_not_required(value: Option<&Value>, _args: &[Value]) -> tera::Result<bool> {
    match value {
        Some(Value::Object(map)) => {
            if let Some(Value::String(req_level)) = map.get("requirement_level") {
                if req_level == "required" {
                    return Ok(false)
                }
            }
        }
        _ => {}
    }
    return Ok(true)
}