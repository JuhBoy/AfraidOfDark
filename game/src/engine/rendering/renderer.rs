use crate::{
    engine::{
        logging::{consts, logs_traits::LoggerBase},
        utils::app_settings::WindowSettings,
    },
    WindowMode,
};
use glfw::{Action, Context, Glfw, GlfwReceiver, Key, PWindow, WindowEvent};
use std::{collections::VecDeque, rc::Rc};
use glfw::ffi::glfwWindowHint;
use crate::engine::rendering::gfx_device::GfxDevice;
use crate::engine::rendering::opengl::GfxDeviceOpengl;

use super::{gfx_device::RenderCommand, renderer_storage::RendererStorage, shaders::ShaderInfo};

pub type OnWindowResizedCb = dyn FnMut(&mut glfw::Window, i32, i32);
pub type RenderCmdHd = u128;

extern crate gl;
extern crate glfw;

pub struct MeshInfo {
    pub file_path: Option<String>,
    pub count: u8,
    pub vertices_set: Option<Vec<Vec<f32>>>
}

pub struct RenderRequest {
    pub vertex_info: ShaderInfo,
    pub fragment_info: ShaderInfo,
    pub mesh_info: MeshInfo,
}

#[derive(Clone, Copy, PartialEq)]
enum RenderState { Opened, Closed }

pub struct Renderer {
    rendering_state: RenderState,
    rendering_queue: VecDeque<RenderCommand>,

    pub instance: Glfw,
    pub window: PWindow,
    pub events: GlfwReceiver<(f64, WindowEvent)>,
    pub log: Rc<dyn LoggerBase>,
    pub on_window_resized: Option<fn(i32, i32)>,

    pub gfx_device: Option<Box<GfxDevice>>,
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
            rendering_state: RenderState::Closed,
            rendering_queue: VecDeque::new(),

            instance,
            window,
            events,
            log,
            gfx_device: Option::from(Box::new(GfxDevice::new(
                Rc::from(GfxDeviceOpengl::default())
            ))),
            on_window_resized: None,
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

    // This is a WIP
    pub fn new_render_command(&mut self, render_req: RenderRequest) -> RenderCmdHd {
        if self.rendering_state == RenderState::Opened {
            println!("Rendering frame has already started, can't add a render command");
            return 0u128;
        }

        let gfx_device = self.gfx_device.as_ref().expect("Graphic device not allocated");

        let vertex_shad = gfx_device.shader_from_file(&render_req.vertex_info).expect("Panic!");
        let fragment_shad = gfx_device.shader_from_file(&render_req.fragment_info).expect("Panic!");

        let shad_mod = gfx_device.new_shader_module(vertex_shad, fragment_shad);
        let buff_mod = gfx_device.alloc_buffer(RendererStorage::load(&render_req.mesh_info), Option::from(false));

        let cmd: RenderCommand = gfx_device.build_command(shad_mod, buff_mod);
        let cmd_hd = cmd.handle;

        self.rendering_queue.push_back(cmd);

        cmd_hd
    }

    pub fn render(&mut self, _delta_time: f32) {
        // Manage inputs there
        self.rendering_state = RenderState::Opened;

        // Render here
        let gfx_device = self.gfx_device.as_ref().expect("Graphic device not allocated");
        gfx_device.clear();

        // rendering_pass. WIP -> will be multithreaded at end
        while !self.rendering_queue.is_empty() { 
            if let Some(command) = self.rendering_queue.pop_front() { 
                gfx_device.use_shader_module(&command.shader_module);
                gfx_device.draw_command(&command);
            }
        }

        self.poll_events();
        self.window.swap_buffers();

        self.rendering_state = RenderState::Closed;
    }
}
