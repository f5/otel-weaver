// SPDX-License-Identifier: Apache-2.0

//! Example 1

use crate::otel::tracer::{ErrorOptionalAttributes, Status};

mod otel;
mod meter;
mod logger;


fn main() {

    let span = otel::tracer::Tracer::start_http_request("test");
    span.url_scheme_attr("https");
    span.client_port_attr(443);
    span.error_event(Some(ErrorOptionalAttributes {
        exception_type: None,
        exception_message: None,
        exception_stacktrace: None,
    }));
    span.status(Status::Ok);
    span.end();
}