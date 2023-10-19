// SPDX-License-Identifier: Apache-2.0

//! Render span.

use crate::search::schema::{attributes, tags};
use ratatui::prelude::{Color, Line, Style};
use ratatui::text::Span;
use ratatui::widgets::Paragraph;

/// Render a span details.
pub fn widget(span: Option<&weaver_schema::span::Span>) -> Paragraph {
    match span {
        Some(span) => {
            let mut text = vec![
                Line::from(vec![
                    Span::styled("Type      : ", Style::default().fg(Color::Yellow)),
                    Span::raw("Span (schema)"),
                ]),
                Line::from(vec![
                    Span::styled("Name      : ", Style::default().fg(Color::Yellow)),
                    Span::raw(&span.span_name),
                ]),
            ];

            if let Some(kind) = span.kind.as_ref() {
                text.push(Line::from(vec![
                    Span::styled("Kind      : ", Style::default().fg(Color::Yellow)),
                    Span::raw(format!("{:?}", kind)),
                ]));
            }

            attributes::append_lines(span.attributes.as_slice(), &mut text);

            if !span.events.is_empty() {
                text.push(Line::from(Span::styled(
                    "Events    : ",
                    Style::default().fg(Color::Yellow),
                )));
                for event in span.events.iter() {
                    text.push(Line::from(Span::raw(format!("- {} ", event.event_name))));
                }
            }

            if !span.links.is_empty() {
                text.push(Line::from(Span::styled(
                    "Links     : ",
                    Style::default().fg(Color::Yellow),
                )));
                for link in span.links.iter() {
                    text.push(Line::from(Span::raw(format!("- {} ", link.link_name))));
                }
            }

            tags::append_lines(span.tags.as_ref(), &mut text);

            Paragraph::new(text).style(Style::default().fg(Color::Gray))
        }
        None => Paragraph::new(vec![Line::default()]),
    }
}
