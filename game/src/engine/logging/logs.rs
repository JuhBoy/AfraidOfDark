use crate::engine::logging::logs_traits::LoggerBase;

#[derive(Debug)]
pub struct Logger {
    pub log_type: &'static str,
}

impl LoggerBase for Logger {
    fn info(&self, info: &str) {
        print!("{}", info);
    }
}
