use serde::{Deserialize, Serialize};
use semconv::attribute::Attribute;
use crate::event::Event;
use crate::link::Link;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Span {
    pub id: String,
    #[serde(default)]
    pub attributes: Vec<Attribute>,
    #[serde(default)]
    pub events: Vec<Event>,
    #[serde(default)]
    pub links: Vec<Link>,
}