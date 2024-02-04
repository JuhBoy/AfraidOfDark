use crate::engine::logging::logs_traits::LoggerBase;
use term;

#[derive(Debug)]
pub struct Logger {
    pub log_type: String,
}

impl LoggerBase for Logger {
    fn info(&self, category: &str, message: &str) {
        let mut t = term::stdout().unwrap();

        match t.fg(term::color::BRIGHT_GREEN) {
            Ok(_) => {
                let info = format!("[{}]: {}", category, message);
                println!("{}", info);
            }
            _ => {}
        };
    }

    fn error(&self, category: &str, error: &str) {
        let mut t = term::stdout().unwrap();

        match t.fg(term::color::BRIGHT_RED) {
            Ok(_) => {
                let line = format!("[{}]: {}", category, error);
                println!("{}", line);
            }
            _ => {}
        };
    }
}
