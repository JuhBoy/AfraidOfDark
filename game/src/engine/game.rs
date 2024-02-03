pub mod runtime {
    use crate::engine::logging::{logs::Logger, logs_traits::LoggerBase};

    pub struct App<'a> {
        name: String,
        logs: &'a dyn LoggerBase
    }

    impl App<'_> {
        pub fn new(name: &str) -> Self {
            Self {
                name: String::from(name),
                logs: &Logger { log_type: "toto" },
            }
        }

        pub fn set_logger(&mut self, logger: dyn LoggerBase) {
            self.logs = logger;
        }

        fn run() {
            // impl loop
        }
    }
}
