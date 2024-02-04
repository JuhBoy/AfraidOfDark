pub trait LoggerBase {
    fn info(&self, category: &str, message: &str);
    fn error(&self, category: &str, error: &str);
}
