// SPDX-License-Identifier: Apache-2.0

//! A schema specification.

use serde::{Deserialize, Serialize};

use crate::event::Event;
use crate::instrumentation_library::InstrumentationLibrary;
use crate::metric_group::MetricGroup;
use crate::resource::Resource;
use crate::resource_events::ResourceEvents;
use crate::resource_metrics::ResourceMetrics;
use crate::resource_spans::ResourceSpans;
use crate::span::Span;
use crate::tags::Tags;
use crate::univariate_metric::UnivariateMetric;

/// Definition of the telemetry schema for an application or a library.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
pub struct SchemaSpec {
    /// A set of tags for the schema.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Tags>,
    /// A common resource specification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<Resource>,
    /// The instrumentation library specification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instrumentation_library: Option<InstrumentationLibrary>,
    /// A resource metrics specification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_metrics: Option<ResourceMetrics>,
    /// A resource events specification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_events: Option<ResourceEvents>,
    /// A resource spans specification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_spans: Option<ResourceSpans>,
}

impl SchemaSpec {
    /// Returns a metric by name or None if not found.
    pub fn metric(&self, name: &str) -> Option<&UnivariateMetric> {
        self.resource_metrics
            .as_ref()
            .map_or(None, |resource_metrics| resource_metrics.metric(name))
    }

    /// Returns a metric group by name or None if not found.
    pub fn metric_group(&self, name: &str) -> Option<&MetricGroup> {
        self.resource_metrics
            .as_ref()
            .map_or(None, |resource_metrics| resource_metrics.metric_group(name))
    }

    /// Returns a vector of metrics.
    pub fn metrics(&self) -> Vec<&UnivariateMetric> {
        self.resource_metrics
            .as_ref()
            .map_or(Vec::<&UnivariateMetric>::new(), |resource_metrics| {
                resource_metrics.metrics()
            })
    }

    /// Returns a vector of metric groups.
    pub fn metric_groups(&self) -> Vec<&MetricGroup> {
        self.resource_metrics
            .as_ref()
            .map_or(Vec::<&MetricGroup>::new(), |resource_metrics| {
                resource_metrics.metric_groups()
            })
    }

    /// Returns a vector over the events.
    pub fn events(&self) -> Vec<&Event> {
        self.resource_events
            .as_ref()
            .map_or(Vec::<&Event>::new(), |resource_events| {
                resource_events.events()
            })
    }

    /// Returns a slice of spans.
    pub fn spans(&self) -> Vec<&Span> {
        self.resource_spans
            .as_ref()
            .map_or(Vec::<&Span>::new(), |resource_spans| resource_spans.spans())
    }

    /// Returns an event by name or None if not found.
    pub fn event(&self, event_name: &str) -> Option<&Event> {
        self.resource_events
            .as_ref()
            .map_or(None, |resource_events| resource_events.event(event_name))
    }

    /// Returns a span by name or None if not found.
    pub fn span(&self, span_name: &str) -> Option<&Span> {
        self.resource_spans
            .as_ref()
            .map_or(None, |resource_spans| resource_spans.span(span_name))
    }
}
