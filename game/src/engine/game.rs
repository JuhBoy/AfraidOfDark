pub mod runtime {
    use crate::engine::{
        logging::{logs::Logger, logs_traits::LoggerBase, consts},
        rendering::renderer::Renderer,
        utils::app_settings::ApplicationSettings,
    };
    use std::rc::Rc;

    pub struct App {
        _name: String,
        logs: Rc<dyn LoggerBase>,
        app_settings: ApplicationSettings,

        // Base systems
        renderer: Option<Renderer>,
    }

    impl App {
        pub fn new(name: &str) -> Self {
            Self {
                _name: String::from(name),
                logs: Rc::new(Logger { log_type: String::from(name) }),
                app_settings: ApplicationSettings::default(),
                renderer: Option::None,
            }
        }

        pub fn with_appsettings(settings: ApplicationSettings) -> Self {
            Self {
                _name: settings.app_name.clone(),
                logs: Rc::new(Logger { log_type: String::from(&settings.app_name) }),
                app_settings: settings,
                renderer: Option::None,
            }
        }

        fn warm(&mut self) {
            let s: &ApplicationSettings = &self.app_settings;
            let renderer: Renderer = Renderer::init_with_glfw(&s.window, self.logs.clone());
            self.renderer = Option::from(renderer);
        }

        pub fn run(&mut self) {
            self.warm();

            let renderer: &mut Renderer;

            match self.renderer.as_mut() {
                None => {
                    self.logs.error(consts::ENGINE_CATEGORY, "Renderer engine system could not be loaded.");
                    panic!();
                },
                Some(r) => renderer = r
            };

            while !renderer.window.should_close() {
                renderer.poll_events();
            }
        }
    }
}
