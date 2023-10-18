// SPDX-License-Identifier: Apache-2.0

//! Render semantic convention attributes.

use ratatui::prelude::{Color, Line, Span, Style};
use ratatui::widgets::Paragraph;
use weaver_semconv::attribute::Attribute;

pub fn widget(attribute: Option<&Attribute>) -> Paragraph {
    match attribute {
        Some(attribute) => {
            let text = vec![
                Line::from(vec![
                    Span::styled("Type   : ", Style::default().fg(Color::Yellow)),
                    Span::raw("Attribute"),
                ]),
                Line::from(vec![
                    Span::styled("Id     : ", Style::default().fg(Color::Yellow)),
                    Span::raw(attribute.id()),
                ]),
                Line::from(vec![
                    Span::styled("Brief  : ", Style::default().fg(Color::Yellow)),
                    Span::raw(attribute.brief()),
                ]),
                Line::from(vec![
                    Span::styled("Note   : ", Style::default().fg(Color::Yellow)),
                    Span::raw(attribute.note()),
                ]),
            ];
            Paragraph::new(text).style(Style::default().fg(Color::Gray))
        }
        None => Paragraph::new(vec![Line::default()])
    }
}