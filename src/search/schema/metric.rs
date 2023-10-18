// SPDX-License-Identifier: Apache-2.0

//! Render metric.

use ratatui::prelude::{Color, Line, Style};
use ratatui::text::Span;
use ratatui::widgets::Paragraph;

/// Render a metric details.
pub fn widget(metric: Option<&weaver_schema::univariate_metric::UnivariateMetric>) -> Paragraph {
    match metric {
        Some(metric) => {
            let text = vec![
                Line::from(vec![
                    Span::styled("Type      : ", Style::default().fg(Color::Yellow)),
                    Span::raw("Metric (schema)"),
                ]),
                Line::from(vec![
                    Span::styled("Name      : ", Style::default().fg(Color::Yellow)),
                    Span::raw(metric.name()),
                ]),
                Line::from(vec![
                    Span::styled("Brief     : ", Style::default().fg(Color::Yellow)),
                    Span::raw(metric.brief()),
                ]),
                Line::from(vec![
                    Span::styled("Note      : ", Style::default().fg(Color::Yellow)),
                    Span::raw(metric.note()),
                ]),
            ];

            Paragraph::new(text).style(Style::default().fg(Color::Gray))
        }
        None => Paragraph::new(vec![Line::default()])
    }
}