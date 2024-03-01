use crate::{
    engine::{
        logging::{consts, logs_traits::LoggerBase},
        utils::app_settings::WindowSettings,
    },
    WindowMode,
};
use glfw::{ffi::glfwInit, Action, Context, Glfw, GlfwReceiver, Key, PWindow, WindowEvent};
use std::{collections::{HashMap, VecDeque}, rc::Rc, sync::Arc};
use glfw::ffi::glfwWindowHint;
use crate::engine::rendering::gfx_device::GfxDevice;
use crate::engine::rendering::opengl::GfxDeviceOpengl;

use super::{gfx_device::RenderCommand, gfx_opengl_shaders::GfxOpenGLShaderApi, renderer_storage::RendererStorage, shaders::{Material, ShaderInfo, ShaderType}};

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
    pub mesh_info: MeshInfo,
    pub material: Arc<Material>
}

#[derive(Clone, Copy, PartialEq)]
enum RenderState { Opened, Closed }

pub struct Renderer {
    rendering_state: RenderState,
    rendering_store: RendererStorage,

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
            rendering_store: RendererStorage { render_command_storage: HashMap::new(), renderer_queue: VecDeque::new() },

            instance,
            window,
            events,
            log,
            gfx_device: Option::from(Box::new(GfxDevice::new(
                Rc::from(GfxDeviceOpengl::default()),
                Rc::from(GfxOpenGLShaderApi::default())
            ))),
            on_window_resized: None,
        }
    }

    pub fn warm(&mut self) {
        self.window.set_cursor_pos_polling(true);
        self.window.set_key_polling(true);

        // Load all function pointers from the graphic driver
        gl::load_with(|procname: &str| self.window.get_proc_address(procname));

        let callback = self.on_window_resized.clone();

        self.window.set_size_callback(move |window: &mut glfw::Window, width: i32, height: i32| {
            if let Some(callback) = callback {
                callback(width, height);
            }

            #[cfg(debug_assertions)] { 
                let (w_factor, h_factor) = window.get_content_scale();
                println!("Window screen coords resized: {}x{} (scale factor {}x{})", width, height, w_factor, h_factor);
            }

            unsafe {
                let (scaled_width, scaled_height) = window.get_framebuffer_size();
                gl::Viewport(0, 0, scaled_width, scaled_height);
            }
        });
        self.window.make_current();

        unsafe {
            glfwInit();
            glfwWindowHint(glfw::ffi::CONTEXT_VERSION_MAJOR, 3);
            glfwWindowHint(glfw::ffi::CONTEXT_VERSION_MINOR, 3);
            glfwWindowHint(glfw::ffi::OPENGL_PROFILE, glfw::ffi::OPENGL_CORE_PROFILE);

            #[cfg(target_os = "macos")]
            glfwWindowHint(glfw::ffi::OPENGL_FORWARD_COMPAT, glfw::ffi::TRUE);

            let (width, height) = self.window.get_framebuffer_size();
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
    pub fn create_render_command(&mut self, render_req: RenderRequest) -> RenderCmdHd {
        if self.rendering_state == RenderState::Opened {
            println!("Rendering frame has already started, can't add a render command");
            return 0u128;
        }

        let gfx_device = self.gfx_device.as_deref_mut().expect("Graphic device not allocated");

        let vert_default = ShaderInfo::default(ShaderType::Vertex);
        let frag_default = ShaderInfo::default(ShaderType::Fragment);

        let vert_info = render_req.material.shaders.vertex.as_ref().unwrap_or(&vert_default);
        let frag_info = render_req.material.shaders.fragment.as_ref().unwrap_or(&frag_default);

        let vs_content = self.rendering_store.load_shader_content(vert_info).expect("Boom!");
        let fs_content = self.rendering_store.load_shader_content(frag_info).expect("Boom!");

        let vs_hdl = gfx_device.alloc_shader(vs_content, ShaderType::Vertex);
        let fs_hdl = gfx_device.alloc_shader(fs_content, ShaderType::Fragment);

        let shader_module = gfx_device.new_shader_module(vs_hdl, fs_hdl);
        let buffer_module = gfx_device.alloc_buffer(RendererStorage::load(&render_req.mesh_info), Option::from(false));

        let command: RenderCommand = gfx_device.build_command(shader_module, buffer_module);
        let command_handle = self.rendering_store.store_command(command, true);

        command_handle
    }

    pub fn enqueue_cmd_for_current_frame(&mut self, handle: RenderCmdHd) {
        self.rendering_store.add_to_frame_queue(handle);
    }

    pub fn get_command(&self, handle: RenderCmdHd) -> Rc<RenderCommand> {
        self.rendering_store.get_ref(handle)
    }

    pub fn remove_render_command(&mut self, handle: RenderCmdHd) {
        self.rendering_store.remove_render_command(handle)
    }

    pub fn render(&mut self, _delta_time: f32) {
        // Manage inputs there
        self.rendering_state = RenderState::Opened;

        // Render here
        let gfx_device = self.gfx_device.as_ref().expect("Graphic device not allocated");
        gfx_device.clear();

        // rendering_pass. WIP -> will be multithreaded at end
        let rendering_queue = &mut self.rendering_store.renderer_queue;
        while !rendering_queue.is_empty() {
            if let Some(command) = rendering_queue.pop_front() {
                gfx_device.use_shader_module(&command.shader_module);
                gfx_device.draw_command(&command);
            }
        }

        self.window.swap_buffers();
        self.poll_events();

        self.rendering_state = RenderState::Closed;
    }
}
