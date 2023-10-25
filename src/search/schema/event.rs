// SPDX-License-Identifier: Apache-2.0

//! Utility functions to index and render events.

use crate::search::schema::{attribute, attributes, tags};
use crate::search::{ColorConfig, DocFields};
use ratatui::prelude::{Color, Line, Style};
use ratatui::text::Span;
use ratatui::widgets::Paragraph;
use tantivy::{doc, IndexWriter};
use weaver_schema::TelemetrySchema;

/// Build index for events.
pub fn index(schema: &TelemetrySchema, fields: &DocFields, index_writer: &mut IndexWriter) {
    for event in schema.events() {
        index_writer
            .add_document(doc!(
                fields.path => format!("schema/event/{}", event.event_name),
                fields.brief => "",
                fields.note => ""
            ))
            .expect("Failed to add document");
        attribute::index_schema_attribute(
            event.attributes.iter(),
            &format!("schema/event/{}", event.event_name),
            fields,
            index_writer,
        );
    }
}

/// Render a span details.
pub fn widget<'a>(
    event: Option<&'a weaver_schema::event::Event>,
    provenance: &'a str,
    colors: &'a ColorConfig,
) -> Paragraph<'a> {
    match event {
        Some(event) => {
            let mut text = vec![
                Line::from(vec![
                    Span::styled("Type      : ", Style::default().fg(colors.label)),
                    Span::raw("Event (schema)"),
                ]),
                Line::from(vec![
                    Span::styled("Name      : ", Style::default().fg(colors.label)),
                    Span::raw(&event.event_name),
                ]),
                Line::from(vec![
                    Span::styled("Domain    : ", Style::default().fg(colors.label)),
                    Span::raw(&event.domain),
                ]),
            ];

            attributes::append_lines(event.attributes.as_slice(), &mut text, colors);
            tags::append_lines(event.tags.as_ref(), &mut text, colors);

            // Provenance
            text.push(Line::from(""));
            text.push(Line::from(Span::styled(
                "Provenance: ",
                Style::default().fg(colors.label),
            )));
            text.push(Line::from(provenance));

            Paragraph::new(text).style(Style::default().fg(Color::Gray))
        }
        None => Paragraph::new(vec![Line::default()]),
    }
}
