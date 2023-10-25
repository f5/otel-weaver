// SPDX-License-Identifier: Apache-2.0

//! Attribute rendering.

use crate::search::theme::ThemeConfig;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use weaver_semconv::attribute::{Attribute, BasicRequirementLevel, RequirementLevel};

/// Append attributes to the text.
pub fn append_lines(attributes: &[Attribute], text: &mut Vec<Line>, theme: &ThemeConfig) {
    if !attributes.is_empty() {
        text.push(Line::from(Span::styled(
            "Attributes: ",
            Style::default().fg(theme.label),
        )));
        for attr in attributes.iter() {
            if let Attribute::Id {
                id,
                r#type,
                requirement_level,
                ..
            } = attr
            {
                let mut properties = vec![format!("type={}", r#type)];
                if let RequirementLevel::Basic(BasicRequirementLevel::Required) = requirement_level
                {
                    properties.push("required".to_string());
                }
                let properties = if properties.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", properties.join(", "))
                };
                text.push(Line::from(Span::raw(format!("- {}{}", id, properties))));
            }
        }
    }
}
