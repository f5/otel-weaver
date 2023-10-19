// SPDX-License-Identifier: Apache-2.0

//! Render examples

use ratatui::prelude::{Color, Line, Span, Style};
use weaver_semconv::attribute::Examples;

/// Append examples to the text.
pub fn append_lines(examples: &Examples, text: &mut Vec<Line>) {
    text.push(Line::from(Span::styled(
        "Examples: ",
        Style::default().fg(Color::Yellow),
    )));
    match examples {
        Examples::Int(v) => text.push(Line::from(Span::raw(format!("- {}", v)))),
        Examples::Double(v) => text.push(Line::from(Span::raw(format!("- {}", v)))),
        Examples::String(v) => text.push(Line::from(Span::raw(format!("- {}", v)))),
        Examples::Ints(vals) => {
            for v in vals.iter() {
                text.push(Line::from(Span::raw(format!("- {}", v))));
            }
        }
        Examples::Doubles(vals) => {
            for v in vals.iter() {
                text.push(Line::from(Span::raw(format!("- {}", v))));
            }
        }
        Examples::Strings(vals) => {
            for v in vals.iter() {
                text.push(Line::from(Span::raw(format!("- {}", v))));
            }
        }
    }
}
