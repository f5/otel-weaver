// SPDX-License-Identifier: Apache-2.0

//! Tags rendering.

use crate::search::theme::ThemeConfig;
use ratatui::prelude::{Line, Span, Style};
use weaver_schema::tags::Tags;

/// Append tags to the text.
pub fn append_lines<'a>(tags: Option<&'a Tags>, text: &mut Vec<Line>, theme: &'a ThemeConfig) {
    if let Some(tags) = tags {
        if tags.is_empty() {
            return;
        }
        text.push(Line::from(Span::styled(
            "Tags      : ",
            Style::default().fg(theme.label),
        )));
        for (k, v) in tags.iter() {
            text.push(Line::from(Span::raw(format!("  - {}={} ", k, v))));
        }
    }
}
