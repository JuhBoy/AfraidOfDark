use crate::engine::rendering::gfx_device::GfxDevice;
use crate::engine::rendering::opengl::GfxDeviceOpengl;
use crate::{
    engine::{
        inputs::keyboard::Keyboard, logging::logs_traits::LoggerBase,
        utils::app_settings::WindowSettings,
    },
    WindowMode,
};
use glfw::ffi::glfwWindowHint;
use glfw::{ffi::glfwInit, Action, Context, Glfw, GlfwReceiver, Key, PWindow, WindowEvent};
use glm::Vector4;
use std::cell::{Ref, RefMut};
use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
    sync::{Arc, Mutex},
};

use super::components::{Rect, RenderUpdate};
use super::gfx_device::BufferModule;
use super::renderer_helpers::{
    COLOR_MASK, compute_gfx_viewport_rect, get_material_changes, TEXTURE_MASK, MaterialUpdateMask,
};
use super::{
    components::{BufferSettings, FrameBuffer, RenderRequest, RenderState},
    gfx_device::{RenderCommand, ShaderModule},
    gfx_opengl_shaders::GfxOpenGLShaderApi,
    renderer_storage::RendererStorage,
    shaders::{Material, ShaderInfo, ShaderType},
};

pub type OnWindowResizedCb = dyn FnMut(&mut glfw::Window, i32, i32);
pub type RenderCmdHd = u128;

extern crate gl;
extern crate glfw;

pub struct Renderer {
    keyboard_inputs: Arc<Mutex<Keyboard>>,
    rendering_state: RenderState,
    rendering_store: RendererStorage,
    viewport_rect: RefCell<glm::Vector4<f32>>, // x, y, width, height (Range is [0; 1])

    main_framebuffer: Option<FrameBuffer>,
    screen_shader_module: Option<ShaderModule>,
    screen_quad_buffer: Option<BufferModule>,

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
            )
            .expect("Failed to create window");

        Self {
            keyboard_inputs: Arc::from(Mutex::from(Keyboard::new())),
            rendering_state: RenderState::Closed,
            rendering_store: RendererStorage {
                render_command_storage: HashMap::new(),
                renderer_queue: VecDeque::new(),
            },
            viewport_rect: RefCell::new(glm::vec4(0.0, 0.0, 1.0, 1.0)),

            main_framebuffer: None,
            screen_shader_module: None,
            screen_quad_buffer: None,

            instance,
            window,
            events,
            log,
            gfx_device: Option::from(Box::new(GfxDevice::new(
                Rc::from(GfxDeviceOpengl::default()),
                Rc::from(GfxOpenGLShaderApi::default()),
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
        let final_x: f32 = scaled_width as f32 * vp_borrow.x;
        let final_y: f32 = scaled_height as f32 * vp_borrow.y;
        let final_w: f32 = scaled_width as f32 * vp_borrow.z;
        let final_h: f32 = scaled_height as f32 * vp_borrow.w;
        let rect: Rect<u32> = Rect {
            x: final_x as u32,
            y: final_y as u32,
            width: final_w as u32,
            height: final_h as u32,
        };

        let device: &Box<GfxDevice> = self
            .gfx_device
            .as_ref()
            .expect("Graphic device not allocated");
        device.set_update_viewport_callback(&mut self.window, self.viewport_rect.clone());
        device.update_viewport(rect);

        // Alloc the main framebuffer for post process, it will blit to the screen buffer
        let frame_buffer: FrameBuffer = device.alloc_framebuffer(scaled_width, scaled_height);

        // allocate base shaders and programs to show the main framebuffer quad to the whole screen
        let vertex_info = ShaderInfo {
            file_name: String::from("framebuffer_vertex.shader"),
            shader_type: ShaderType::Vertex,
            load_type: super::shaders::ShaderLoadType::OnDemand,
        };
        let fragment_info = ShaderInfo {
            file_name: String::from("framebuffer_fragment.shader"),
            shader_type: ShaderType::Fragment,
            load_type: super::shaders::ShaderLoadType::OnDemand,
        };
        let vert_source = self
            .rendering_store
            .load_shader_content(&vertex_info)
            .expect("Failed to create shader");
        let frag_source = self
            .rendering_store
            .load_shader_content(&fragment_info)
            .expect("Failed to create shader");
        let vert_hdl = device.alloc_shader(vert_source, ShaderType::Vertex);
        let frag_hdl = device.alloc_shader(frag_source, ShaderType::Fragment);
        let shader_module = device.alloc_shader_module(vert_hdl, frag_hdl, &Material::new());

        // Alloc the quand buffer used to draw the entire viewport
        let screen_quad: BufferModule = device.alloc_buffer(
            vec![RendererStorage::load_2d_quad()],
            vec![],
            BufferSettings {
                keep_vertices: false,
                vertex_size: 2,
                uvs_size: 2,
            },
        );

        // store the main framebuffer to self
        self.main_framebuffer = Option::from(frame_buffer);
        self.screen_shader_module = Option::from(shader_module);
        self.screen_quad_buffer = Option::from(screen_quad);
    }

    pub fn get_keyboard_inputs(&self) -> Arc<Mutex<Keyboard>> {
        self.keyboard_inputs.clone()
    }

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

        let gfx_device = self
            .gfx_device
            .as_deref_mut()
            .expect("Graphic device not allocated");

        let vert_default = ShaderInfo::default(ShaderType::Vertex);
        let frag_default = ShaderInfo::default(ShaderType::Fragment);

        let vert_info = render_req
            .material
            .shaders
            .vertex
            .as_ref()
            .unwrap_or(&vert_default);
        let frag_info = render_req
            .material
            .shaders
            .fragment
            .as_ref()
            .unwrap_or(&frag_default);

        let vs_content = self
            .rendering_store
            .load_shader_content(vert_info)
            .expect("Boom!");
        let fs_content = self
            .rendering_store
            .load_shader_content(frag_info)
            .expect("Boom!");

        let vs_hdl = gfx_device.alloc_shader(vs_content, ShaderType::Vertex);
        let fs_hdl = gfx_device.alloc_shader(fs_content, ShaderType::Fragment);

        let mut shader_module =
            gfx_device.alloc_shader_module(vs_hdl, fs_hdl, &render_req.material);
        let buffer_settings = BufferSettings {
            keep_vertices: false,
            vertex_size: 3,
            uvs_size: 2,
        };
        let buffer_module = gfx_device.alloc_buffer(
            RendererStorage::load(&render_req.mesh_info),
            vec![RendererStorage::get_quad_indices()],
            buffer_settings,
        );

        if let Some(tex_name) = render_req.material.main_texture.as_ref() {
            let texture = self.rendering_store.load_texture(&tex_name).ok().unwrap();
            let texture_handle: u32 = gfx_device.alloc_texture(shader_module.self_handle, &texture);

            const DEFAULT_TEXTURE_IDX: i32 = 0;
            gfx_device
                .shader_api
                .set_texture_unit(shader_module.self_handle, DEFAULT_TEXTURE_IDX);
            shader_module.texture_handles.push(texture_handle);
        }

        gfx_device.shader_api.set_attribute_color(
            shader_module.self_handle,
            "surface_color",
            shader_module.material.color,
        );

        let command: RenderCommand = gfx_device.build_command(shader_module, buffer_module);
        let command_handle = self.rendering_store.store_command(command, true);

        command_handle
    }

    pub fn update_render_command(&mut self, update_req: RenderUpdate) -> bool {
        let mut command: RefMut<RenderCommand> =
            self.rendering_store.get_mut_ref(update_req.render_cmd);

        let mut update_mask: MaterialUpdateMask = 0u8;

        // check changes in material properties
        if !update_req.material.is_none() {
            let current: &Material = &command.shader_module.material;
            let updated: &Material = update_req.material.as_ref().unwrap();
            update_mask = get_material_changes(current, updated);
        }

        if update_mask == 0 {
            return false;
        }

        let gpu: &mut GfxDevice = self.gfx_device.as_deref_mut().expect("gfx_device not init");

        if (update_mask & COLOR_MASK) != 0 {
            let new_color: Vector4<f32> = update_req.material.as_ref().unwrap().color;

            gpu.shader_api.set_attribute_color(
                command.shader_module.self_handle,
                "surface_color",
                new_color,
            );

            command.shader_module.material.color = new_color;
        }

        if (update_mask & TEXTURE_MASK) != 0 {
            let texture_name: &String = update_req
                .material
                .as_ref()
                .unwrap()
                .main_texture
                .as_ref()
                .unwrap();

            match self.rendering_store.load_texture(texture_name) {
                Ok(texture) => {
                    let texture_handle: u32 =
                        gpu.alloc_texture(command.shader_module.self_handle, &texture);
                    gpu.update_texture(&mut command.shader_module, texture_handle);
                }
                Err(err) => {
                    panic!("[Renderr]: failed to load texture {}", err)
                }
            }
        }

        true
    }

    pub fn enqueue_cmd_for_current_frame(&mut self, handle: RenderCmdHd) {
        self.rendering_store.add_to_frame_queue(handle);
    }

    pub fn get_command(&self, handle: RenderCmdHd) -> Ref<RenderCommand> {
        self.rendering_store.get_ref(handle)
    }

    pub fn remove_render_command(&mut self, handle: RenderCmdHd) {
        self.rendering_store.remove_render_command(handle)
    }

    pub fn update_viewport(&mut self, x: f32, y: f32, width: f32, height: f32) {
        // Update the viewport on CPU side for now
        let mut vp_borrow = self.viewport_rect.borrow_mut();

        vp_borrow.x = x;
        vp_borrow.y = y;
        vp_borrow.z = width;
        vp_borrow.w = height;
    }

    pub fn render(&mut self, _delta_time: f32) {
        // Manage inputs there
        self.rendering_state = RenderState::Opened;

        // Render here
        let gfx_device = self
            .gfx_device
            .as_ref()
            .expect("Graphic device not allocated");

        // Updates to default viewport before drawing the scene into the main buffer texture
        let default_viewport: glm::Vector4<f32> = Vector4 {
            x: 0f32,
            y: 0f32,
            z: 1f32,
            w: 1f32,
        };
        let viewport: Rect<u32> = compute_gfx_viewport_rect(&default_viewport, &self.window);
        gfx_device.update_viewport(viewport);

        gfx_device.use_framebuffer(Option::from(&self.main_framebuffer));
        gfx_device.clear();

        // rendering_pass. WIP -> will be multithreaded at end
        let rendering_queue = &mut self.rendering_store.renderer_queue;
        while !rendering_queue.is_empty() {
            if let Some(command) = rendering_queue.pop_front() {
                let comand_ref: Ref<'_, RenderCommand> = command.borrow();
                gfx_device.use_shader_module(&comand_ref.shader_module);
                gfx_device.draw_command(&comand_ref);
            }
        }

        // Back to screen buffer
        gfx_device.use_framebuffer(None);
        gfx_device.clear();

        let normalized_screen_viewport = self.viewport_rect.clone().into_inner();
        let pixel_screen_viewport: Rect<u32> =
            compute_gfx_viewport_rect(&normalized_screen_viewport, &self.window);
        gfx_device.update_viewport(pixel_screen_viewport);

        gfx_device.use_shader_module(self.screen_shader_module.as_ref().unwrap());
        gfx_device.blit_main_framebuffer(
            self.screen_quad_buffer.as_ref().unwrap(),
            self.main_framebuffer.as_ref().unwrap(),
        );

        self.window.swap_buffers();
        self.poll_events();

        self.rendering_state = RenderState::Closed;
    }
}
