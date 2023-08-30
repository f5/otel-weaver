use serde::{Deserialize, Serialize};
use semconv::attribute::Attribute;
use crate::span::Span;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ResourceSpans {
    #[serde(default)]
    pub attributes: Vec<Attribute>,
    #[serde(default)]
    pub spans: Vec<Span>,
}