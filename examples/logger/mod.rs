// SPDX-License-Identifier: Apache-2.0

//! Generated code for the logger API.

use std::error::Error;

#[derive(Default)]
pub struct HttpLogAttrs<'a> {
    network_protocol_name: Option<&'a str>,
    network_protocol_version: Option<&'a str>,
}

pub struct Logger {}

impl Logger {
    pub fn http<'a>(&self,
                    server_address: &str,
                    server_port: i64,
                    http_request_method: &str,
                    http_response_status_code: i64,
                    url_scheme: &str,
                    body: &str,
                    opt_attrs: Option<HttpLogAttrs<'a>>,
    ) {}
}

