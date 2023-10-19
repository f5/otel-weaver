// SPDX-License-Identifier: Apache-2.0

//! Render semantic convention attributes.

use crate::search::semconv::attributes;
use ratatui::prelude::{Color, Line, Span, Style};
use ratatui::widgets::Paragraph;
use weaver_semconv::metric::Metric;

pub fn widget(metric: Option<&Metric>) -> Paragraph {
    match metric {
        Some(metric) => {
            let mut text = vec![
                Line::from(vec![
                    Span::styled("Type      : ", Style::default().fg(Color::Yellow)),
                    Span::raw("Metric"),
                ]),
                Line::from(vec![
                    Span::styled("Name      : ", Style::default().fg(Color::Yellow)),
                    Span::raw(metric.name.clone()),
                ]),
                Line::from(vec![
                    Span::styled("Instrument: ", Style::default().fg(Color::Yellow)),
                    Span::raw(format!("{:?}", metric.instrument)),
                ]),
                Line::from(vec![
                    Span::styled("Brief     : ", Style::default().fg(Color::Yellow)),
                    Span::raw(metric.brief.clone()),
                ]),
                Line::from(vec![
                    Span::styled("Note      : ", Style::default().fg(Color::Yellow)),
                    Span::raw(metric.note.clone()),
                ]),
                Line::from(vec![
                    Span::styled("Unit      : ", Style::default().fg(Color::Yellow)),
                    Span::raw(metric.unit.clone().unwrap_or_default()),
                ]),
            ];
            attributes::append_lines(metric.attributes.as_slice(), &mut text);
            Paragraph::new(text).style(Style::default().fg(Color::Gray))
        }
        None => Paragraph::new(vec![Line::default()]),
    }
}
