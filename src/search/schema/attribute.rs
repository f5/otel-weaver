// SPDX-License-Identifier: Apache-2.0

//! Attribute rendering.

use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use weaver_schema::attribute::Attribute;

/// Append attributes to the text.
pub fn append_lines(attributes: &[Attribute], text: &mut Vec<Line>) {
    if !attributes.is_empty() {
        text.push(Line::from(Span::styled("Attributes: ", Style::default().fg(Color::Yellow))));
        for attr in attributes.iter() {
            text.push(Line::from(Span::raw(format!("- {} ", attr.id()))));
        }
    }
}