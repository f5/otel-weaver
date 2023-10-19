// SPDX-License-Identifier: Apache-2.0

//! Tags rendering.

use ratatui::prelude::{Color, Line, Span, Style};
use weaver_schema::tags::Tags;

/// Append tags to the text.
pub fn append_lines(tags: Option<&Tags>, text: &mut Vec<Line>) {
    if let Some(tags) = tags {
        if tags.is_empty() {
            return;
        }
        text.push(Line::from(Span::styled(
            "Tags      : ",
            Style::default().fg(Color::Yellow),
        )));
        for (k, v) in tags.iter() {
            text.push(Line::from(Span::raw(format!("  - {}={} ", k, v))));
        }
    }
}
