// SPDX-License-Identifier: Apache-2.0

//! Render semantic convention attributes.

use ratatui::prelude::{Color, Line, Span, Style};
use ratatui::widgets::Paragraph;

use weaver_semconv::attribute::Attribute;
use weaver_semconv::AttributeWithProvenance;

use crate::search::semconv::examples;

pub fn widget(attribute: Option<&AttributeWithProvenance>) -> Paragraph {
    match attribute.as_ref() {
        Some(AttributeWithProvenance {
                 attribute: Attribute::Id {
                     id,
                     r#type,
                     brief,
                     examples,
                     tag,
                     requirement_level,
                     sampling_relevant,
                     note,
                     stability,
                     deprecated,
                 }, provenance
             }) => {
            let mut text = vec![
                Line::from(vec![
                    Span::styled("Id   : ", Style::default().fg(Color::Yellow)),
                    Span::raw(id),
                ]),
                Line::from(vec![
                    Span::styled("Type : ", Style::default().fg(Color::Yellow)),
                    Span::raw(format!("{}", r#type)),
                ]),
            ];

            // Tag
            if let Some(tag) = tag {
                text.push(Line::from(vec![
                    Span::styled("Tag  : ", Style::default().fg(Color::Yellow)),
                    Span::raw(tag),
                ]));
            }

            // Brief
            if !brief.trim().is_empty() {
                text.push(Line::from(""));
                text.push(Line::from(Span::styled("Brief: ", Style::default().fg(Color::Yellow))));
                text.push(Line::from(brief.as_str()));
            }

            // Note
            if !note.trim().is_empty() {
                text.push(Line::from(""));
                text.push(Line::from(Span::styled("Note : ", Style::default().fg(Color::Yellow))));
                text.push(Line::from(note.as_str()));
            }

            // Requirement Level
            text.push(Line::from(""));
            text.push(Line::from(vec![
                Span::styled("Requirement Level: ", Style::default().fg(Color::Yellow)),
                Span::raw(format!("{}", requirement_level)),
            ]));

            if let Some(sampling_relevant) = sampling_relevant {
                text.push(Line::from(vec![
                    Span::styled("Sampling Relevant: ", Style::default().fg(Color::Yellow)),
                    Span::raw(sampling_relevant.to_string()),
                ]));
            }

            if let Some(stability) = stability {
                text.push(Line::from(vec![
                    Span::styled("Stability: ", Style::default().fg(Color::Yellow)),
                    Span::raw(format!("{}", stability)),
                ]));
            }

            if let Some(deprecated) = deprecated {
                text.push(Line::from(vec![
                    Span::styled("Deprecated: ", Style::default().fg(Color::Yellow)),
                    Span::raw(format!("{}", deprecated)),
                ]));
            }

            if let Some(examples) = examples {
                examples::append_lines(examples, &mut text);
            }

            // Provenance
            text.push(Line::from(""));
            text.push(Line::from(Span::styled("Provenance: ", Style::default().fg(Color::Yellow))));
            text.push(Line::from(provenance.as_str()));

            Paragraph::new(text).style(Style::default().fg(Color::Gray))
        }
        _ => Paragraph::new(vec![Line::from("Attribute not resolved!")]),
    }
}
