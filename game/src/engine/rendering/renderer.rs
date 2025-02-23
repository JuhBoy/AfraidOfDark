use crate::{
    engine::{
        inputs::keyboard::Keyboard, logging::logs_traits::LoggerBase, utils::app_settings::WindowSettings
    },
    WindowMode,
};
use glfw::{ffi::glfwInit, Action, Context, Glfw, GlfwReceiver, Key, PWindow, WindowEvent};
use std::{cell::RefCell, collections::{HashMap, VecDeque}, rc::Rc, sync::{Arc, Mutex}};
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
    pub material: Material
}

#[derive(Clone, Copy, PartialEq)]
enum RenderState { Opened, Closed }

pub struct Renderer {
		keyboard_inputs: Arc<Mutex<Keyboard>>,
    rendering_state: RenderState,
    rendering_store: RendererStorage,
    viewport_rect: RefCell<glm::Vector4<f32>>, // x, y, width, height (Range is [0; 1])

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

        // Set the OpenGL version to 3.3 - todo! export this in opengl files
        unsafe {
            glfwInit();
            glfwWindowHint(glfw::ffi::CONTEXT_VERSION_MAJOR, 3);
            glfwWindowHint(glfw::ffi::CONTEXT_VERSION_MINOR, 3);

            #[cfg(target_os = "macos")]
            { 
                glfwWindowHint(glfw::ffi::OPENGL_PROFILE, glfw::ffi::OPENGL_CORE_PROFILE);
                glfwWindowHint(glfw::ffi::OPENGL_FORWARD_COMPAT, glfw::ffi::TRUE);
            }
        }

        let (window, events) = instance
            .create_window(
                settings.width,
                settings.height,
                "Game",
                match settings.mode {
                    WindowMode::Windowed => glfw::WindowMode::Windowed,
                    _ => panic!(),
                },
            ).expect("Failed to create window");

        Self {
						keyboard_inputs: Arc::from(Mutex::from(Keyboard::new())),
            rendering_state: RenderState::Closed,
            rendering_store: RendererStorage { render_command_storage: HashMap::new(), renderer_queue: VecDeque::new() },
            viewport_rect: RefCell::new(glm::vec4(0.0, 0.0, 1.0, 1.0)),

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

        let (scaled_width, scaled_height) = self.window.get_framebuffer_size();

        let vp_borrow = self.viewport_rect.borrow();
        let final_x = scaled_width as f32 * vp_borrow.x;
        let final_y = scaled_height as f32 * vp_borrow.y;
        let final_w = scaled_width as f32 * vp_borrow.z;
        let final_h = scaled_height as f32 * vp_borrow.w;

        let device = self.gfx_device.as_ref().expect("Graphic device not allocated");
        device.set_update_viewport_callback(&mut self.window, self.viewport_rect.clone());
        device.update_viewport(final_x as u32, final_y as u32, final_w as u32, final_h as u32);
    }

		pub fn get_keyboard_inputs(&self) -> Arc<Mutex<Keyboard>> { self.keyboard_inputs.clone() }

    pub fn poll_events(&mut self) {
        self.instance.poll_events();
				
				let mut keyboard_inputs = self.keyboard_inputs.lock().unwrap();
				keyboard_inputs.pre_update_states();

        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    self.window.set_should_close(true)
                }
                _ => match event {
                    WindowEvent::Key(k, _scan_code, action, modifier) => {
												keyboard_inputs.update_key_state(k, action, modifier);
                    }
                    _ => {}
                },
            }
        }
    }

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

        let mut shader_module = gfx_device.alloc_shader_module(vs_hdl, fs_hdl, &render_req.material);
        let buffer_module = gfx_device.alloc_buffer(RendererStorage::load(&render_req.mesh_info), vec![RendererStorage::get_quad_indices()], Option::from(false));

        if let Some(tex_name) = render_req.material.main_texture.as_ref() {
            let texture = self.rendering_store.load_texture(&tex_name).ok().unwrap();
            let texture_handle = gfx_device.shader_api.set_texture(shader_module.self_handle, texture, 0);
            shader_module.texture_hadles.push(texture_handle);
            println!("[Texture Loading] New Texture handle: {}", texture_handle)
        }

				gfx_device.shader_api.set_attribute_color(shader_module.self_handle, "surface_color", shader_module.material.color);

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

    pub fn update_viewport(&mut self, x: f32, y: f32, width: f32, height: f32) {
        let mut vp_borrow = self.viewport_rect.borrow_mut();
        vp_borrow.x = x;
        vp_borrow.y = y;
        vp_borrow.z = width;
        vp_borrow.w = height;

        let (scaled_width, scaled_height) = self.window.get_framebuffer_size();

        let final_x = scaled_width as f32 * vp_borrow.x;
        let final_y = scaled_height as f32 * vp_borrow.y;
        let final_w = scaled_width as f32 * vp_borrow.z;
        let final_h = scaled_height as f32 * vp_borrow.w;

        let device = self.gfx_device.as_ref().expect("Graphic device not allocated");
        device.update_viewport(final_x as u32, final_y as u32, final_w as u32, final_h as u32);
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
