use crate::{engine::{logging::{consts, logs_traits::LoggerBase}, utils::app_settings::WindowSettings}, WindowMode};
use glfw::{Context, Glfw, GlfwReceiver, PWindow, WindowEvent, Key, Action};
use std::rc::Rc;
extern crate glfw;

pub struct Renderer {
    pub instance: Glfw,
    pub window: PWindow,
    pub events: GlfwReceiver<(f64, WindowEvent)>,
    pub log: Rc<dyn LoggerBase>
}

impl Renderer {
    pub fn init_with_glfw(settings: &WindowSettings, log: Rc<dyn LoggerBase>) -> Self {
        let mut instance = glfw::init(glfw::fail_on_errors).unwrap();

        let (mut window, events) = instance
            .create_window(
                settings.width,
                settings.height,
                "Game",
                match settings.mode {
                    WindowMode::Windowed => glfw::WindowMode::Windowed,
                    _ => panic!(),
                },
            )
            .expect("Failed to create window");

        window.set_key_polling(true);
        window.make_current();

        Self {
            instance,
            window,
            events,
            log
        }
    }

    pub fn poll_events(&mut self) {
        self.instance.poll_events();

        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                WindowEvent::Key(Key::Escape, _, Action::Press, _) => self.window.set_should_close(true),
                _ => match event {
                    WindowEvent::Key(k, _a, _b, _c) => {
                        let name: String;

                        if let Some(s) = k.get_name() {
                            name = s.to_owned();
                        } else {
                            name = String::from("unknow touch");
                        }

                        self.log.info(consts::ENGINE_RENDERING, &format!("Key not handled by engine {}", name));
                    }
                    _ => {}
                },
            }
        }
    }
}
