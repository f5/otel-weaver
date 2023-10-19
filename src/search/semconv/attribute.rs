// SPDX-License-Identifier: Apache-2.0

//! Render semantic convention attributes.

use ratatui::prelude::{Color, Line, Span, Style};
use ratatui::widgets::Paragraph;

use weaver_semconv::attribute::Attribute;
use crate::search::semconv::examples;

pub fn widget(attribute: Option<&Attribute>) -> Paragraph {
    match attribute.as_ref() {
        Some(Attribute::Id { id, r#type, brief, examples, tag, requirement_level, sampling_relevant, note, stability, deprecated }) => {
            let mut text = vec![
                Line::from(vec![
                    Span::styled("Type   : ", Style::default().fg(Color::Yellow)),
                    Span::raw("Attribute"),
                ]),
                Line::from(vec![
                    Span::styled("Id     : ", Style::default().fg(Color::Yellow)),
                    Span::raw(id),
                ]),
                Line::from(vec![
                    Span::styled("Type   : ", Style::default().fg(Color::Yellow)),
                    Span::raw(format!("{:?}", r#type)),
                ]),
                Line::from(vec![
                    Span::styled("Brief  : ", Style::default().fg(Color::Yellow)),
                    Span::raw(brief),
                ]),
                Line::from(vec![
                    Span::styled("Note   : ", Style::default().fg(Color::Yellow)),
                    Span::raw(note),
                ]),
            ];

            text.push(Line::from(vec![
                Span::styled("Requirement Level: ", Style::default().fg(Color::Yellow)),
                Span::raw(format!("{:?}", requirement_level)),
            ]));

            if let Some(tag) = tag {
                text.push(Line::from(vec![
                    Span::styled("Tag    : ", Style::default().fg(Color::Yellow)),
                    Span::raw(tag),
                ]));
            }

            if let Some(sampling_relevant) = sampling_relevant {
                text.push(Line::from(vec![
                    Span::styled("Sampling Relevant: ", Style::default().fg(Color::Yellow)),
                    Span::raw(sampling_relevant.to_string()),
                ]));
            }

            if let Some(stability) = stability {
                text.push(Line::from(vec![
                    Span::styled("Stability: ", Style::default().fg(Color::Yellow)),
                    Span::raw(format!("{:?}", stability)),
                ]));
            }

            if let Some(deprecated) = deprecated {
                text.push(Line::from(vec![
                    Span::styled("Deprecated: ", Style::default().fg(Color::Yellow)),
                    Span::raw(format!("{:?}", deprecated)),
                ]));
            }

            if let Some(examples) = examples {
                examples::append_lines(examples, &mut text);
            }

            Paragraph::new(text).style(Style::default().fg(Color::Gray))
        }
        _ => Paragraph::new(vec![Line::from("Attribute not resolved!")])
    }
}