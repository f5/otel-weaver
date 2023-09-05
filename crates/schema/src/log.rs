// SPDX-License-Identifier: Apache-2.0

//! Log record specification.

use semconv::attribute::Attribute;
use serde::{Deserialize, Serialize};

/// A log record specification.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Log {
    /// The name of the log record.
    pub id: String,
    /// The type of body of the log record.
    pub body: BodyType,
    /// The attributes of the log record.
    #[serde(default)]
    pub attributes: Vec<Attribute>,
}

/// The type of body of a log record.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum BodyType {
    /// A boolean body.
    Boolean(bool),
    /// An integer body.
    Int(i64),
    /// A double body.
    Double(f64),
    /// A string body.
    String(String),
    /// A boolean array body.
    #[serde(rename = "boolean[]")]
    Booleans(Vec<String>),
    /// An integer array body.
    #[serde(rename = "int[]")]
    Ints(Vec<String>),
    /// A double array body.
    #[serde(rename = "double[]")]
    Doubles(Vec<String>),
    /// A string array body.
    #[serde(rename = "string[]")]
    Strings(Vec<String>),
}
