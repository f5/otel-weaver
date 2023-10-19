// SPDX-License-Identifier: Apache-2.0

//! Render metric.

use ratatui::prelude::{Color, Line, Style};
use ratatui::text::Span;
use ratatui::widgets::Paragraph;

use weaver_schema::metric_group::{Metric, MetricGroup};

use crate::search::schema::{attributes, tags};

/// Render a metric details.
pub fn widget(metric_group: Option<&MetricGroup>) -> Paragraph {
    match metric_group {
        Some(metric_group) => {
            let mut text = vec![Line::from(vec![
                Span::styled("Type      : ", Style::default().fg(Color::Yellow)),
                Span::raw("Metric Group (schema)"),
            ])];

            text.push(Line::from(vec![
                Span::styled("Name      : ", Style::default().fg(Color::Yellow)),
                Span::raw(metric_group.id.clone()),
            ]));

            attributes::append_lines(metric_group.attributes.as_slice(), &mut text);

            if !metric_group.metrics.is_empty() {
                text.push(Line::from(Span::styled(
                    "Metrics   : ",
                    Style::default().fg(Color::Yellow),
                )));
                for metric in metric_group.metrics.iter() {
                    if let Metric::Metric { name, tags, .. } = metric {
                        let mut properties = vec![];
                        if let Some(tags) = tags {
                            if !tags.is_empty() {
                                let mut pairs = vec![];
                                for (k, v) in tags.iter() {
                                    pairs.push(format!("{}={}", k, v));
                                }
                                properties.push(format!("tags=[{}]", pairs.join(",")));
                            }
                        }
                        let properties = if properties.is_empty() {
                            String::new()
                        } else {
                            format!(" ({})", properties.join(", "))
                        };
                        text.push(Line::from(Span::raw(format!("- {}{}", name, properties))));
                    }
                }
            }

            tags::append_lines(metric_group.tags.as_ref(), &mut text);

            Paragraph::new(text).style(Style::default().fg(Color::Gray))
        }
        None => Paragraph::new(vec![Line::default()]),
    }
}
