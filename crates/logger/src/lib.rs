// SPDX-License-Identifier: Apache-2.0

//! A generic logger that can be used to log messages to the console.

#![deny(missing_docs)]
#![deny(clippy::print_stdout)]
#![deny(clippy::print_stderr)]

/// A generic logger that can be used to log messages to the console.
#[derive(Default)]
pub struct Logger<'a> {
    logger: paris::Logger<'a>,
    debug_level: u8,
}

impl<'a> Logger<'a> {
    /// Creates a new logger.
    pub fn new(debug_level: u8) -> Self {
        Logger {
            logger: paris::Logger::new(),
            debug_level,
        }
    }

    /// Logs an trace message (only with debug enabled).
    pub fn trace(&mut self, message: &str) -> &mut Self {
        if self.debug_level > 0 {
            self.logger.log(message);
        }
        self
    }

    /// Logs an info message.
    pub fn info(&mut self, message: &str) -> &mut Self {
        self.logger.info(message);
        self
    }

    /// Logs a warning message.
    pub fn warn(&mut self, message: &str) -> &mut Self {
        self.logger.warn(message);
        self
    }

    /// Logs an error message.
    pub fn error(&mut self, message: &str) -> &mut Self {
        self.logger.newline(1);
        self.logger.error(message);
        self
    }

    /// Logs a success message.
    pub fn success(&mut self, message: &str) -> &mut Self {
        self.logger.success(message);
        self
    }

    /// Logs a newline.
    pub fn newline(&mut self, count: usize) -> &mut Self {
        self.logger.newline(count);
        self
    }

    /// Indents the logger.
    pub fn indent(&mut self, count: usize) -> &mut Self {
        self.logger.indent(count);
        self
    }

    /// Stops a loading message.
    pub fn done(&mut self) {
        self.logger.done();
    }

    /// Adds a style to the logger.
    pub fn add_style(&mut self, name: &str, styles: Vec<&'a str>) -> &mut Self {
        self.logger.add_style(name, styles);
        self
    }

    /// Logs a loading message with a spinner.
    pub fn loading(&mut self, message: &str) -> &mut Self {
        self.logger.loading(message);
        self
    }

    /// Forces the logger to not print a newline for the next message.
    pub fn same(&mut self) -> &mut Self {
        self.logger.same();
        self
    }

    /// Logs a message without icon.
    pub fn log(&mut self, message: &str) -> &mut Self {
        self.logger.log(message);
        self
    }
}
