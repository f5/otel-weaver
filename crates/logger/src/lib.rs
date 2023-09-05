//! A generic logger that can be used to log messages to the console.

#![deny(missing_docs)]
#[deny(clippy::print_stdout)]

/// A generic logger that can be used to log messages to the console.
#[derive(Default)]
pub struct Logger<'a> {
    logger: paris::Logger<'a>,
}

impl<'a> Logger<'a> {
    /// Creates a new logger.
    pub fn new() -> Self {
        Default::default()
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
