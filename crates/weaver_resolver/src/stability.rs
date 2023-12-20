// SPDX-License-Identifier: Apache-2.0

//! Functions to resolve a semantic convention stability field.

use weaver_semconv::stability::Stability;

pub fn resolve_stability(
    stability: &Option<Stability>,
) -> Option<weaver_resolved_schema::catalog::Stability> {
    stability.as_ref().map(|stability| match stability {
        Stability::Deprecated => weaver_resolved_schema::catalog::Stability::Deprecated,
        Stability::Experimental => weaver_resolved_schema::catalog::Stability::Experimental,
        Stability::Stable => weaver_resolved_schema::catalog::Stability::Stable,
    })
}
