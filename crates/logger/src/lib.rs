pub struct Logger<'a> {
    logger: paris::Logger<'a>,
}

impl<'a> Logger<'a> {
    pub fn new() -> Self {
        Self {
            logger: paris::Logger::new(),
        }
    }

    pub fn info(&mut self, message: &str) -> &mut Self {
        self.logger.info(message);
        self
    }

    pub fn warn(&mut self, message: &str) -> &mut Self {
        self.logger.warn(message);
        self
    }

    pub fn error(&mut self, message: &str) -> &mut Self {
        self.logger.newline(1);
        self.logger.error(message);
        self
    }

    pub fn success(&mut self, message: &str) -> &mut Self {
        self.logger.success(message);
        self
    }

    pub fn newline(&mut self, count: usize) -> &mut Self {
        self.logger.newline(count);
        self
    }

    pub fn indent(&mut self, count: usize) -> &mut Self {
        self.logger.indent(count);
        self
    }

    pub fn done(&mut self) {
        self.logger.done();
    }

    pub fn add_style(&mut self, name: &str, styles: Vec<&'a str>) -> &mut Self {
        self.logger.add_style(name, styles);
        self
    }

    pub fn loading(&mut self, message: &str) -> &mut Self {
        self.logger.loading(message);
        self
    }

    pub fn same(&mut self) -> &mut Self {
        self.logger.same();
        self
    }

    pub fn log(&mut self, message: &str) -> &mut Self {
        self.logger.log(message);
        self
    }
}