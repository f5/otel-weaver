// SPDX-License-Identifier: Apache-2.0

//! Render metric.

use ratatui::prelude::{Color, Line, Style};
use ratatui::text::Span;
use ratatui::widgets::Paragraph;

use crate::search::schema::{attributes, tags};
use weaver_schema::univariate_metric::UnivariateMetric;

/// Render a metric details.
pub fn widget(metric: Option<&UnivariateMetric>) -> Paragraph {
    match metric {
        Some(metric) => {
            let mut text = vec![Line::from(vec![
                Span::styled("Type      : ", Style::default().fg(Color::Yellow)),
                Span::raw("Metric (schema)"),
            ])];

            if let UnivariateMetric::Metric {
                name,
                brief,
                note,
                attributes,
                instrument,
                unit,
                tags,
            } = metric
            {
                text.push(Line::from(vec![
                    Span::styled("Name      : ", Style::default().fg(Color::Yellow)),
                    Span::raw(name),
                ]));
                text.push(Line::from(vec![
                    Span::styled("Brief     : ", Style::default().fg(Color::Yellow)),
                    Span::raw(brief),
                ]));
                text.push(Line::from(vec![
                    Span::styled("Note      : ", Style::default().fg(Color::Yellow)),
                    Span::raw(note),
                ]));

                attributes::append_lines(attributes.as_slice(), &mut text);

                if let Some(instrument) = instrument {
                    text.push(Line::from(vec![
                        Span::styled("Instrument: ", Style::default().fg(Color::Yellow)),
                        Span::raw(format!("{:?}", instrument)),
                    ]));
                }

                if let Some(unit) = unit {
                    text.push(Line::from(vec![
                        Span::styled("Unit      : ", Style::default().fg(Color::Yellow)),
                        Span::raw(unit),
                    ]));
                }

                tags::append_lines(tags.as_ref(), &mut text);
            }
            Paragraph::new(text).style(Style::default().fg(Color::Gray))
        }
        None => Paragraph::new(vec![Line::default()]),
    }
}
