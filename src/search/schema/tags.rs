// SPDX-License-Identifier: Apache-2.0

//! Tags rendering.

use crate::search::ColorConfig;
use ratatui::prelude::{Line, Span, Style};
use weaver_schema::tags::Tags;

/// Append tags to the text.
pub fn append_lines<'a>(tags: Option<&'a Tags>, text: &mut Vec<Line>, colors: &'a ColorConfig) {
    if let Some(tags) = tags {
        if tags.is_empty() {
            return;
        }
        text.push(Line::from(Span::styled(
            "Tags      : ",
            Style::default().fg(colors.label),
        )));
        for (k, v) in tags.iter() {
            text.push(Line::from(Span::raw(format!("  - {}={} ", k, v))));
        }
    }
}
