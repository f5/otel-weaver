// SPDX-License-Identifier: Apache-2.0

//!

use std::error::Error;
use crate::otel::tracer::Status;

pub struct Tracer {}

impl Tracer {
    pub fn start(
        name: &str,

    ) -> Span {
        Span::default()
    }
}

#[derive(Default)]
pub struct Span {}


/// Optional attributes for `error` event.
#[derive(Default)]
pub struct ErrorOptionalAttributes {
    /// The type of the exception (its fully-qualified class name, if
    /// applicable). The dynamic type of the exception should be preferred over
    /// the static type in languages that support it.
    ///
    /// # Examples:
    /// * java.net.ConnectException
    /// * OSError
    pub exception_type: Option<&str>,

    /// The exception message.
    ///
    /// # Examples:
    /// * Division by zero
    /// * Can't convert 'int' object to str implicitly
    pub exception_message: Option<&str>,

    /// A stacktrace as a string in the natural representation for the language
    /// runtime. The representation is to be determined and documented by each
    /// language SIG.
    ///
    /// # Examples:
    /// * Exception in thread "main" java.lang.RuntimeException: Test exception
    ///  at com.example.GenerateTrace.methodB(GenerateTrace.java:13)
    ///  at com.example.GenerateTrace.methodA(GenerateTrace.java:9)
    ///  at com.example.GenerateTrace.main(GenerateTrace.java:5)
    pub exception_stacktrace: Option<&str>,

}

impl Span {
    /// Optional span attributes
    /// Server address - domain name if available without reverse DNS lookup,
    /// otherwise IP address or Unix domain socket name.
    /// When observed from the client side, and when communicating through an
    /// intermediary, `server.address` SHOULD represent
    /// the server address behind any intermediaries (e.g. proxies) if it's
    /// available.
    /// # Examples:
    /// * example.com
    pub fn server_address_attr(&self, value: &str) {}

    /// Server port number
    /// When observed from the client side, and when communicating through an
    /// intermediary, `server.port` SHOULD represent the server port behind any
    /// intermediaries (e.g. proxies) if it's available.

    pub fn server_port_attr(&self, value: i64) {}

    /// Server address of the socket connection - IP address or Unix domain
    /// socket name.
    /// When observed from the client side, this SHOULD represent the immediate
    /// server peer address.
    /// When observed from the server side, this SHOULD represent the physical
    /// server address.
    /// # Examples:
    /// * 10.5.3.2
    pub fn server_socket_address_attr(&self, value: &str) {}

    /// Server port number of the socket connection.
    /// When observed from the client side, this SHOULD represent the immediate
    /// server peer port.
    /// When observed from the server side, this SHOULD represent the physical
    /// server port.

    pub fn server_socket_port_attr(&self, value: i64) {}

    /// Client address - domain name if available without reverse DNS lookup,
    /// otherwise IP address or Unix domain socket name.
    /// When observed from the server side, and when communicating through
    /// an intermediary, `client.address` SHOULD represent the client address
    /// behind any intermediaries (e.g. proxies) if it's available.
    /// # Examples:
    /// * /tmp/my.sock
    /// * 10.1.2.80
    pub fn client_address_attr(&self, value: &str) {}

    /// Client port number.
    /// When observed from the server side, and when communicating through an
    /// intermediary, `client.port` SHOULD represent the client port behind any
    /// intermediaries (e.g. proxies) if it's available.

    pub fn client_port_attr(&self, value: i64) {}

    /// Client address of the socket connection - IP address or Unix domain
    /// socket name.
    /// When observed from the server side, this SHOULD represent the immediate
    /// client peer address.
    /// When observed from the client side, this SHOULD represent the physical
    /// client address.
    /// # Examples:
    /// * /tmp/my.sock
    /// * 127.0.0.1
    pub fn client_socket_address_attr(&self, value: &str) {}

    /// Client port number of the socket connection.
    /// When observed from the server side, this SHOULD represent the immediate
    /// client peer port.
    /// When observed from the client side, this SHOULD represent the physical
    /// client port.

    pub fn client_socket_port_attr(&self, value: i64) {}

    /// The [URI scheme](https://www.rfc-editor.org/rfc/rfc3986#section-3.1)
    /// component identifying the used protocol.
    ///
    /// # Examples:
    /// * https
    /// * ftp
    /// * telnet
    pub fn url_scheme_attr(&self, value: &str) {}


    pub fn error_event(&self, optional_attrs: Option<crate::otel::tracer::ErrorOptionalAttributes>) {}

    pub fn status(&self, status: Status) {}
    pub fn error(&self, err: &dyn Error) {}

    pub fn end(self) {}
}

