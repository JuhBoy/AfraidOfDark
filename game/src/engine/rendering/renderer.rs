use crate::{
    engine::{
        logging::{consts, logs_traits::LoggerBase},
        utils::app_settings::WindowSettings,
    },
    WindowMode,
};
use gl::types::*;
use glfw::{Action, Context, Glfw, GlfwReceiver, Key, PWindow, WindowEvent};
use std::rc::Rc;
use glfw::ffi::glfwWindowHint;

pub type OnWindowResizedCb = dyn FnMut(&mut glfw::Window, i32, i32);

extern crate gl;
extern crate glfw;

pub struct Renderer {
    pub instance: Glfw,
    pub window: PWindow,
    pub events: GlfwReceiver<(f64, WindowEvent)>,
    pub log: Rc<dyn LoggerBase>,
    pub on_window_resized: Option<fn(i32, i32)>
}

impl Renderer {
    pub fn init_with_glfw(settings: &WindowSettings, log: Rc<dyn LoggerBase>) -> Self {
        let mut instance = glfw::init(glfw::fail_on_errors).unwrap();

        let (window, events) = instance
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

        Self {
            instance,
            window,
            events,
            log,
            on_window_resized: None
        }
    }

    pub fn warm(&mut self) {
        self.window.set_cursor_pos_polling(true);
        self.window.set_key_polling(true);

        // Load all function pointers from the graphic driver
        gl::load_with(|s: &str| self.window.get_proc_address(s));

        let callback = self.on_window_resized.clone();

        self.window.set_size_callback(move |_w: &mut glfw::Window, width: i32, height: i32| {
            if let Some(callback) = callback {
                callback(width, height);
            }

            #[cfg(debug_assertions)]
            println!("Window resized: {} {}", width, height);

            unsafe {
                gl::Viewport(0, 0, width, height);
            }
        });
        self.window.make_current();

        unsafe {
            glfwWindowHint(glfw::ffi::CONTEXT_VERSION_MAJOR, 3);
            glfwWindowHint(glfw::ffi::CONTEXT_VERSION_MINOR, 3);
            glfwWindowHint(glfw::ffi::OPENGL_PROFILE, glfw::ffi::OPENGL_CORE_PROFILE);

            #[cfg(target_os = "macos")]
            glfwWindowHint(glfw::ffi::OPENGL_FORWARD_COMPAT, glfw::ffi::TRUE);

            let (width, height) = self.window.get_size();
            gl::Viewport(0, 0, width, height);
        }
    }

    pub fn poll_events(&mut self) {
        self.instance.poll_events();

        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    self.window.set_should_close(true)
                }
                _ => match event {
                    WindowEvent::Key(k, _a, _b, _c) => {
                        let name: String;

                        if let Some(s) = k.get_name() {
                            name = s.to_owned();
                        } else {
                            name = String::from("unknow touch");
                        }

                        self.log.info(
                            consts::ENGINE_RENDERING,
                            &format!("Key not handled by engine {}", name),
                        );
                    }
                    _ => {}
                },
            }
        }
    }

    pub fn render(&mut self, _delta_time: f32) {
        unsafe {
            // Manage inputs there

            gl::ClearColor(0.5f32,0.2f32,0.3f32,1.0f32);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            // Render here

            self.poll_events();
            self.window.swap_buffers();
        }
    }
}
