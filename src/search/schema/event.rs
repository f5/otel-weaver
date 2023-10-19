// SPDX-License-Identifier: Apache-2.0

//! Render span.

use ratatui::prelude::{Color, Line, Style};
use ratatui::text::Span;
use ratatui::widgets::Paragraph;
use crate::search::schema::{attribute, tags};

/// Render a span details.
pub fn widget(event: Option<&weaver_schema::event::Event>) -> Paragraph {
    match event {
        Some(event) => {
            let mut text = vec![
                Line::from(vec![
                    Span::styled("Type      : ", Style::default().fg(Color::Yellow)),
                    Span::raw("Event (schema)"),
                ]),
                Line::from(vec![
                    Span::styled("Name      : ", Style::default().fg(Color::Yellow)),
                    Span::raw(&event.event_name),
                ]),
                Line::from(vec![
                    Span::styled("Domain    : ", Style::default().fg(Color::Yellow)),
                    Span::raw(&event.domain),
                ]),
            ];

            attribute::append_lines(event.attributes.as_slice(), &mut text);
            tags::append_lines(event.tags.as_ref(), &mut text);

            Paragraph::new(text).style(Style::default().fg(Color::Gray))
        }
        None => Paragraph::new(vec![Line::default()])
    }
}