// SPDX-License-Identifier: Apache-2.0

//! A generic logger that can be used to log messages to the console.

#![deny(missing_docs)]
#![deny(clippy::print_stdout)]
#![deny(clippy::print_stderr)]

use std::sync::{Arc, Mutex};

/// A generic logger that can be used to log messages to the console.
/// This logger is thread-safe and can be cloned.
#[derive(Default, Clone)]
pub struct Logger<'a> {
    logger: Arc<Mutex<paris::Logger<'a>>>,
    debug_level: u8,
}

impl<'a> Logger<'a> {
    /// Creates a new logger.
    pub fn new(debug_level: u8) -> Self {
        Logger {
            logger: Arc::new(Mutex::new(paris::Logger::new())),
            debug_level,
        }
    }

    /// Logs an trace message (only with debug enabled).
    pub fn trace(&self, message: &str) -> &Self {
        if self.debug_level > 0 {
            self.logger
                .lock()
                .expect("Failed to lock logger")
                .log(message);
        }
        self
    }

    /// Logs an info message.
    pub fn info(&self, message: &str) -> &Self {
        self.logger
            .lock()
            .expect("Failed to lock logger")
            .info(message);
        self
    }

    /// Logs a warning message.
    pub fn warn(&self, message: &str) -> &Self {
        self.logger
            .lock()
            .expect("Failed to lock logger")
            .warn(message);
        self
    }

    /// Logs an error message.
    pub fn error(&self, message: &str) -> &Self {
        self.logger
            .lock()
            .expect("Failed to lock logger")
            .error(message);
        self
    }

    /// Logs a success message.
    pub fn success(&self, message: &str) -> &Self {
        self.logger
            .lock()
            .expect("Failed to lock logger")
            .success(message);
        self
    }

    /// Logs a newline.
    pub fn newline(&self, count: usize) -> &Self {
        self.logger
            .lock()
            .expect("Failed to lock logger")
            .newline(count);
        self
    }

    /// Indents the logger.
    pub fn indent(&self, count: usize) -> &Self {
        self.logger
            .lock()
            .expect("Failed to lock logger")
            .indent(count);
        self
    }

    /// Stops a loading message.
    pub fn done(&self) {
        self.logger.lock().expect("Failed to lock logger").done();
    }

    /// Adds a style to the logger.
    pub fn add_style(&self, name: &str, styles: Vec<&'a str>) -> &Self {
        self.logger
            .lock()
            .expect("Failed to lock logger")
            .add_style(name, styles);
        self
    }

    /// Logs a loading message with a spinner.
    pub fn loading(&self, message: &str) -> &Self {
        self.logger
            .lock()
            .expect("Failed to lock logger")
            .loading(message);
        self
    }

    /// Forces the logger to not print a newline for the next message.
    pub fn same(&self) -> &Self {
        self.logger.lock().expect("Failed to lock logger").same();
        self
    }

    /// Logs a message without icon.
    pub fn log(&self, message: &str) -> &Self {
        self.logger
            .lock()
            .expect("Failed to lock logger")
            .log(message);
        self
    }
}
