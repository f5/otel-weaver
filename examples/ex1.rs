// SPDX-License-Identifier: Apache-2.0

//! Example 1

use crate::otel::tracers::http_request::{Event, HttpRequestAttrs, HttpRequestOptAttrs, Status};

mod otel;

fn main() {
    let mut span = otel::tracers::http_request::start("test", HttpRequestAttrs {
        url_host: "localhost".to_string(),
    });
    span.attr_url_scheme("https".to_string());
    span.attr_client_port(443);
    span.event(Event::Error {
        exception_type: None,
        exception_message: None,
        exception_stacktrace: None,
    });
    span.status(Status::Ok);
    span.end();

    let mut span = otel::tracers::http_request::start("test", HttpRequestAttrs {
        url_host: "localhost".to_string(),
    });
    span.event(Event::Error {
        exception_type: None,
        exception_message: None,
        exception_stacktrace: None,
    });
    span.status(Status::Ok);
    span.end_with_opt_attrs(HttpRequestOptAttrs {
        url_scheme: Some("https".to_string()),
        client_port: Some(443),
        ..Default::default()
    });

    otel::loggers::http::log(otel::loggers::http::Attrs {
        server_address: Some("localhost".to_string()),
        server_port: Some(443),
        http_response_status_code: Some(200),
        network_protocol_name: Some("http".to_string()),
        network_protocol_version: None,
        url_scheme: None,
    });
}