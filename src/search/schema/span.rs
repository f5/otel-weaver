// SPDX-License-Identifier: Apache-2.0

//! Utility functions to index and render spans.

use crate::search::schema::{attribute, attributes, tags};
use crate::search::DocFields;
use ratatui::prelude::{Color, Line, Style};
use ratatui::text::Span;
use ratatui::widgets::Paragraph;
use tantivy::{doc, IndexWriter};
use weaver_schema::TelemetrySchema;

/// Build index for spans.
pub fn index(schema: &TelemetrySchema, fields: &DocFields, index_writer: &mut IndexWriter) {
    for span in schema.spans() {
        index_writer
            .add_document(doc!(
                fields.path => format!("schema/span/{}", span.span_name),
                fields.brief => "",
                fields.note => ""
            ))
            .expect("Failed to add document");
        attribute::index_schema_attribute(
            span.attributes.iter(),
            &format!("schema/span/{}", span.span_name),
            fields,
            index_writer,
        );
        for event in span.events.iter() {
            index_writer
                .add_document(doc!(
                    fields.path => format!("schema/span/{}/event/{}", span.span_name, event.event_name),
                    fields.brief => "",
                    fields.note => ""
                ))
                .expect("Failed to add document");
            attribute::index_schema_attribute(event.attributes.iter(), &format!("schema/span/{}/event/{}", span.span_name, event.event_name), fields, index_writer);
        }
    }
}

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
