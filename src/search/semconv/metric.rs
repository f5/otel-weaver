// SPDX-License-Identifier: Apache-2.0

//! Render semantic convention attributes.

use ratatui::prelude::{Color, Line, Span, Style};
use ratatui::widgets::Paragraph;

use weaver_semconv::MetricWithProvenance;

use crate::search::semconv::attributes;

pub fn widget(metric: Option<&MetricWithProvenance>) -> Paragraph {
    match metric {
        Some(MetricWithProvenance { metric, provenance }) => {
            let mut text = vec![
                Line::from(vec![
                    Span::styled("Name      : ", Style::default().fg(Color::Yellow)),
                    Span::raw(metric.name.clone()),
                ]),
                Line::from(vec![
                    Span::styled("Instrument: ", Style::default().fg(Color::Yellow)),
                    Span::raw(format!("{:?}", metric.instrument)),
                ]),
                Line::from(vec![
                    Span::styled("Unit      : ", Style::default().fg(Color::Yellow)),
                    Span::raw(metric.unit.clone().unwrap_or_default()),
                ]),
            ];

            // Brief
            if !metric.brief.trim().is_empty() {
                text.push(Line::from(""));
                text.push(Line::from(Span::styled("Brief     : ", Style::default().fg(Color::Yellow))));
                text.push(Line::from(metric.brief.as_str()));
            }

            // Note
            if !metric.note.trim().is_empty() {
                text.push(Line::from(""));
                text.push(Line::from(Span::styled("Note      : ", Style::default().fg(Color::Yellow))));
                text.push(Line::from(metric.note.as_str()));
            }

            attributes::append_lines(metric.attributes.as_slice(), &mut text);

            // Provenance
            text.push(Line::from(""));
            text.push(Line::from(vec![
                Span::styled("Provenance: ", Style::default().fg(Color::Yellow)),
                Span::raw(format!("{}", provenance)),
            ]));

            Paragraph::new(text).style(Style::default().fg(Color::Gray))
        }
        None => Paragraph::new(vec![Line::default()]),
    }
}
