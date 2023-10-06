// SPDX-License-Identifier: Apache-2.0

//! Resolve metric and metric_group

use crate::attribute::{merge_attributes, resolve_attributes};
use crate::Error;
use logger::Logger;
use schema::attribute::to_schema_attributes;
use schema::metric_group::Metric;
use schema::schema_spec::SchemaSpec;
use schema::univariate_metric::UnivariateMetric;
use semconv::SemConvCatalog;
use version::VersionChanges;

/// Resolves metrics and their attributes.
pub fn resolve_metrics(
    log: &mut Logger,
    schema: &mut SchemaSpec,
    sem_conv_catalog: &mut SemConvCatalog,
    version_changes: &VersionChanges,
) -> Result<(), Error> {
    if let Some(metrics) = schema.resource_metrics.as_mut() {
        metrics.attributes = resolve_attributes(
            metrics.attributes.as_ref(),
            &sem_conv_catalog,
            version_changes.metric_attribute_changes(),
        )?;
        for metric in metrics.metrics.iter_mut() {
            if let UnivariateMetric::Ref {
                r#ref,
                attributes,
                tags,
            } = metric
            {
                *attributes = resolve_attributes(
                    attributes,
                    &sem_conv_catalog,
                    version_changes.metric_attribute_changes(),
                )?;
                if let Some(referenced_metric) = sem_conv_catalog.get_metric(r#ref) {
                    let mut inherited_attrs =
                        to_schema_attributes(&referenced_metric.attributes);
                    inherited_attrs = resolve_attributes(
                        &inherited_attrs,
                        &sem_conv_catalog,
                        version_changes.metric_attribute_changes(),
                    )?;
                    let merged_attrs = merge_attributes(attributes, &inherited_attrs);
                    *metric = UnivariateMetric::Metric {
                        name: referenced_metric.name.clone(),
                        brief: referenced_metric.brief.clone(),
                        note: referenced_metric.note.clone(),
                        attributes: merged_attrs,
                        instrument: referenced_metric.instrument.clone(),
                        unit: referenced_metric.unit.clone(),
                        tags: tags.clone(),
                    };
                } else {
                    return Err(Error::FailToResolveMetric {
                        r#ref: r#ref.clone(),
                    });
                }
            }
        }
        for metrics in metrics.metric_groups.iter_mut() {
            metrics.attributes = resolve_attributes(
                metrics.attributes.as_ref(),
                &sem_conv_catalog,
                version_changes.metric_attribute_changes(),
            )?;
            for metric in metrics.metrics.iter_mut() {
                if let Metric::Ref { r#ref, tags } = metric {
                    if let Some(referenced_metric) = sem_conv_catalog.get_metric(r#ref) {
                        let inherited_attrs = referenced_metric.attributes.clone();
                        if !inherited_attrs.is_empty() {
                            log.warn(&format!("Attributes inherited from the '{}' metric will be disregarded. Instead, the common attributes specified for the metric group '{}' will be utilized.", r#ref, metrics.id));
                        }
                        *metric = Metric::Metric {
                            name: referenced_metric.name.clone(),
                            brief: referenced_metric.brief.clone(),
                            note: referenced_metric.note.clone(),
                            attributes: metrics.attributes.clone(),
                            instrument: referenced_metric.instrument.clone(),
                            unit: referenced_metric.unit.clone(),
                            tags: tags.clone(),
                        };
                    } else {
                        return Err(Error::FailToResolveMetric {
                            r#ref: r#ref.clone(),
                        });
                    }
                }
            }
        }
    }
    Ok(())
}
