extern crate gl;
extern crate glfw;
use super::components::{ARGB8Color, RenderUpdate, RenderingCamera, RenderingUpdateState};
use super::gfx_device::BufferModule;
use super::renderer_helpers::{
    compute_gfx_viewport_rect, get_material_changes, get_shader_info_or_default,
    shader_texture_update, MaterialUpdateMask, TextureUpdateReq, COLOR_MASK, TEXTURE_MASK,
    TRANSFORM_MASK,
};
use super::{
    components::{BufferSettings, FrameBuffer, RenderRequest, RenderState},
    gfx_device::{RenderCommand, ShaderModule},
    gfx_opengl_shaders::GfxOpenGLShaderApi,
    renderer_storage::RendererStorage,
    shaders::{Material, ShaderInfo, ShaderType},
};
use crate::engine::ecs::components::Transform;
use crate::engine::rendering::gfx_device::GfxDevice;
use crate::engine::rendering::opengl::GfxDeviceOpengl;
use crate::engine::utils::maths::{compute_projection, compute_trs, compute_view_matrix, Rect};
use crate::{
    engine::{
        inputs::keyboard::Keyboard, logging::logs_traits::LoggerBase,
        utils::app_settings::WindowSettings,
    },
    WindowMode,
};
use glfw::ffi::glfwWindowHint;
use glfw::{ffi::glfwInit, Action, Context, Glfw, GlfwReceiver, Key, PWindow, WindowEvent};
use glm::{Matrix4, Vector4};
use std::cell::{Ref, RefMut};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

pub type OnWindowResizedCb = dyn FnMut(&mut glfw::Window, i32, i32);
pub type RenderCmdHd = usize;

pub struct Renderer {
    keyboard_inputs: Arc<Mutex<Keyboard>>,
    rendering_state: RenderState,
    rendering_store: RendererStorage,
    viewport_normalized: RefCell<Vector4<f32>>, // x, y, width, height (Range is [0; 1])
    window_rect: Rect<u32>,
    main_camera: RenderingCamera,
    updates_state: RenderingUpdateState,

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
            rendering_store: RendererStorage::new(),
            viewport_normalized: RefCell::new(glm::vec4(0.0, 0.0, 1.0, 1.0)),
            window_rect: Rect {
                x: 0,
                y: 0,
                width: settings.width,
                height: settings.height,
            },
            main_camera: RenderingCamera {
                near: 0.1,
                far: 50.0,
                ppu: 380u32,
                clear_color: ARGB8Color::black(),
                transform: Transform::default(),
            },
            updates_state: RenderingUpdateState {
                camera_settings: false,
                camera_transform: false,
            },

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

        // Update viewport pixels just in case it would have changed
        self.window_rect.width = scaled_width as u32;
        self.window_rect.height = scaled_height as u32;

        let device: &Box<GfxDevice> = self
            .gfx_device
            .as_ref()
            .expect("Graphic device not allocated");
        device.set_update_viewport_callback(&mut self.window, self.viewport_normalized.clone());

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
            return 0 as RenderCmdHd;
        }

        let gfx = self
            .gfx_device
            .as_deref_mut()
            .expect("Graphic device not allocated");

        let [vert_info, frag_info] = get_shader_info_or_default(&render_req);
        let vs_content = self
            .rendering_store
            .load_shader_content(&vert_info)
            .expect("[Renderer] Could not load shader");
        let fs_content = self
            .rendering_store
            .load_shader_content(&frag_info)
            .expect("[Renderer] Could not load shader");

        let vs_hdl = gfx.alloc_shader(vs_content, ShaderType::Vertex);
        let fs_hdl = gfx.alloc_shader(fs_content, ShaderType::Fragment);

        let mut shader_module = gfx.alloc_shader_module(vs_hdl, fs_hdl, &render_req.material);

        let buffer_module = gfx.alloc_buffer(
            RendererStorage::load(&render_req.mesh_info),
            vec![RendererStorage::get_quad_indices()],
            BufferSettings::quad_default(),
        );

        if let Some(tex_name) = render_req.material.main_texture.as_ref() {
            let texture_handle: u32;

            // Try to load the gpu handle if possible, otherwise allocate a new texture on the gpu side
            if !self.rendering_store.has_gpu_texture_refs(tex_name) {
                let texture = self.rendering_store.load_texture(&tex_name).ok().unwrap();
                texture_handle = gfx.alloc_texture(shader_module.self_handle, &texture);

                // Store the newly created rendering handle to prevent duplicates allocation
                self.rendering_store
                    .increment_texture_handle(tex_name, texture_handle);
            } else {
                // This can panic if the tex_name key doesn't exit
                texture_handle = self.rendering_store.get_gpu_texture_handle(tex_name);
            }

            const DEFAULT_TEXTURE_IDX: i32 = 0;
            gfx.shader_api
                .set_texture_unit(shader_module.self_handle, DEFAULT_TEXTURE_IDX);
            shader_module.texture_handles.push(texture_handle);
        }

        gfx.shader_api.set_attribute_color(
            shader_module.self_handle,
            "surface_color",
            shader_module.material.color,
        );

        // compute the TRS, View and Proj matrix and forward them to the GPU device
        let trs_matrix: Matrix4<f32> = compute_trs(&render_req.transform);
        let view_matrix: Matrix4<f32> = compute_view_matrix(&self.main_camera.transform);
        let proj_matrix: Matrix4<f32> = compute_projection(&self.main_camera, &self.window_rect);

        gfx.shader_api
            .set_attribute_mat4(shader_module.self_handle, "TRS", &trs_matrix);
        gfx.shader_api
            .set_attribute_mat4(shader_module.self_handle, "VIEW", &view_matrix);
        gfx.shader_api
            .set_attribute_mat4(shader_module.self_handle, "PROJ", &proj_matrix);

        let command: RenderCommand = gfx.build_command(shader_module, buffer_module);
        let command_handle = self.rendering_store.store_command(command, true);

        command_handle
    }

    pub fn update_render_command(&mut self, update_req: RenderUpdate) -> bool {
        let mut update_mask: MaterialUpdateMask = 0u8;

        // check changes in material properties and construct mask
        if !update_req.material.is_none() {
            let command: Ref<RenderCommand> = self.rendering_store.get_ref(update_req.render_cmd);
            let current: &Material = &command.shader_module.material;
            let updated: &Material = update_req.material.as_ref().unwrap();
            update_mask = get_material_changes(current, updated);
        }

        if update_req.transform.is_some() {
            update_mask |= TRANSFORM_MASK;
        }

        if update_mask == 0 {
            return false;
        }

        let gpu: &mut GfxDevice = self.gfx_device.as_deref_mut().expect("gfx_device not init");

        if (update_mask & COLOR_MASK) != 0 {
            let mut command: RefMut<RenderCommand> =
                self.rendering_store.get_mut_ref(update_req.render_cmd);
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

            let texture_handle: Option<u32>;

            if self.rendering_store.has_gpu_texture_refs(texture_name) {
                texture_handle =
                    Option::from(self.rendering_store.get_gpu_texture_handle(texture_name));
            } else {
                match self.rendering_store.load_texture(texture_name) {
                    Ok(texture) => {
                        let shader_hdl = self
                            .rendering_store
                            .get_ref(update_req.render_cmd)
                            .shader_module
                            .self_handle;
                        texture_handle = Option::from(gpu.alloc_texture(shader_hdl, &texture));
                    }
                    Err(err) => {
                        println!(
                            "[Renderer]: Abort texture update. Failed to load texture {}",
                            err
                        );
                        texture_handle = None;
                    }
                }
            }

            if let Some(gpu_handle) = texture_handle {
                shader_texture_update(
                    &mut self.rendering_store,
                    TextureUpdateReq {
                        handle: update_req.render_cmd.clone(),
                        input_texture_handle: Option::from((texture_name.clone(), gpu_handle)),
                    },
                );
            }
        }

        if (update_mask & TRANSFORM_MASK) != 0 {
            let command = self.rendering_store.get_ref(update_req.render_cmd);
            let new_trs_mat4: Matrix4<f32> = compute_trs(update_req.transform.as_ref().unwrap());
            gpu.shader_api.set_attribute_mat4(
                command.shader_module.self_handle,
                "TRS",
                &new_trs_mat4,
            );
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

    pub fn cull(&mut self, handle: RenderCmdHd, value: bool) {
        self.rendering_store.mark_culled(handle, value);
    }

    pub fn get_world_camera_viewport(&self) -> Rect<f32> {
        let position = &self.main_camera.transform.position;
        let window_rect = &self.window_rect;

        Rect {
            x: position.x,
            y: position.y,
            width: window_rect.width as f32 / self.main_camera.ppu as f32,
            height: window_rect.height as f32 / self.main_camera.ppu as f32,
        }
    }

    pub fn update_normalized_viewport(&mut self, x: f32, y: f32, width: f32, height: f32) {
        // Update the viewport on CPU side for now
        let mut vp_borrow = self.viewport_normalized.borrow_mut();

        vp_borrow.x = x;
        vp_borrow.y = y;
        vp_borrow.z = width;
        vp_borrow.w = height;
    }

    pub fn update_camera_settings(&mut self, camera_update: RenderingCamera) {
        self.main_camera = camera_update;
        self.updates_state.camera_settings = true;
    }

    pub fn update_camera_transform(&mut self, transform: Transform) {
        self.main_camera.transform = transform;
        self.updates_state.camera_transform = true;
    }

    pub fn render(&mut self, _delta_time: f32) {
        self.rendering_state = RenderState::Opened;

        let gfx_device = self
            .gfx_device
            .as_ref()
            .expect("Graphic device not allocated");

        // Updates to default viewport before drawing the scene into the main buffer texture
        let default_viewport: Vector4<f32> = Vector4 {
            x: 0f32,
            y: 0f32,
            z: 1f32,
            w: 1f32,
        };
        let viewport: Rect<u32> = compute_gfx_viewport_rect(&default_viewport, &self.window);
        gfx_device.update_viewport(viewport);

        gfx_device.use_framebuffer(Option::from(&self.main_framebuffer));
        gfx_device.clear(self.main_camera.clear_color);

        // rendering_pass. WIP -> will be multithreaded at end
        let mut rendering_queue = self.rendering_store.renderer_queue.borrow_mut();
        while !rendering_queue.is_empty() {
            if let Some(cmd_ptr) = rendering_queue.pop_front() {
                let command: Ref<RenderCommand> = cmd_ptr.borrow();

                gfx_device.use_shader_module(&command.shader_module);

                // only updates VIEW/PROJ matrix if the camera transform/settings changed
                if self.updates_state.camera_transform {
                    gfx_device.shader_api.set_attribute_mat4(
                        command.shader_module.self_handle,
                        "VIEW",
                        &compute_view_matrix(&self.main_camera.transform),
                    );
                }
                if self.updates_state.camera_settings {
                    gfx_device.shader_api.set_attribute_mat4(
                        command.shader_module.self_handle,
                        "PROJ",
                        &compute_projection(&self.main_camera, &self.window_rect),
                    );
                }
                
                if self.rendering_store.is_culled(command.handle) {
                    continue;
                }

                gfx_device.draw_command(&command);
            }
        }
        drop(rendering_queue);

        // Back to screen buffer
        gfx_device.use_framebuffer(None);
        gfx_device.clear(self.main_camera.clear_color);

        let normalized_screen_viewport = self.viewport_normalized.clone().into_inner();
        let pixel_screen_viewport: Rect<u32> =
            compute_gfx_viewport_rect(&normalized_screen_viewport, &self.window);
        gfx_device.update_viewport(pixel_screen_viewport);

        gfx_device.use_shader_module(self.screen_shader_module.as_ref().unwrap());
        gfx_device.blit_main_framebuffer(
            self.screen_quad_buffer.as_ref().unwrap(),
            self.main_framebuffer.as_ref().unwrap(),
        );

        // Release all dangling textures
        self.rendering_store.iter_dangling_textures(|name, hdl| {
            println!("[Renderer] Release textures {} {}", name, hdl);
            gfx_device.release_texture(hdl);
        });

        // Reset the various states for the current frame
        self.rendering_store.reset_frame();
        self.updates_state.reset();

        self.window.swap_buffers();

        self.rendering_state = RenderState::Closed;
    }
}
