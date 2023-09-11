// SPDX-License-Identifier: Apache-2.0

//! Generator code for the meter API.

use std::error::Error;

#[derive(Default)]
pub struct ServerRequestDurationAttrs<'a> {
    network_protocol_name: Option<&'a str>,
    network_protocol_version: Option<&'a str>,
}

#[derive(Default)]
pub struct HttpUnivariateMetricProvider {}

impl HttpUnivariateMetricProvider {
    pub fn server_request_duration<'a>(&self,
                                       server_address: &str,
                                       server_port: i64,
                                       http_request_method: &str,
                                       http_response_status_code: i64,
                                       url_scheme: &str,
                                       value: i64,
                                       opt_attrs: Option<ServerRequestDurationAttrs<'a>>,
    ) {}
}

#[derive(Default)]
pub struct HttpAttrs<'a> {
    network_protocol_name: Option<&'a str>,
    network_protocol_version: Option<&'a str>,
}

#[derive(Default)]
pub struct HttpMultivariateMetricProvider {}

impl HttpMultivariateMetricProvider {
    pub fn report<'a>(&self,
                      server_address: &str,
                      server_port: i64,
                      http_request_method: &str,
                      http_response_status_code: i64,
                      url_scheme: &str,
                      http_server_request_duration: i64,
                      http_server_active_requests: i64,
                      http_server_request_size: i64,
                      http_server_response_size: i64,
                      opt_attrs: Option<HttpAttrs<'a>>,
    ) {}
}