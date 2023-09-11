// SPDX-License-Identifier: Apache-2.0

//! Generated code for the tracer API.

use std::error::Error;

pub enum Status {
    Unset,
    Error,
    Ok,
}

pub struct Tracer {}

impl Tracer {
    pub fn start_http_request(name: &str) -> HttpRequestSpan {
        HttpRequestSpan::default()
    }
}

#[derive(Default)]
pub struct HttpRequestSpan {}

impl HttpRequestSpan {
    pub fn error_event(
        &self,
        exception_type: &str,
        exception_message: &str,
        exception_stacktrace: &str,
    ) {}

    pub fn error_event_with_ts(
        &self,
        timestamp: i64,
        exception_type: &str,
        exception_message: &str,
        exception_stacktrace: &str,
    ) {}

    pub fn status(&self, status: Status) {}

    pub fn error(&self, err: &dyn Error) {}

    /// An optional attributes
    pub fn url_scheme_attr(&self, url_scheme: &str) {}

    pub fn end(self,
               server_address: &str,
               server_port: i64,
               server_socket_address: &str,
               server_socket_port: i64,
               client_address: &str,
               client_port: i64,
               client_socket_address: &str,
               client_socket_port: i64,
               url_scheme: &str,
    ) {}
}
