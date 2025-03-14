use super::{
    components::{BufferSettings, FrameBuffer, Rect},
    renderer::RenderCmdHd,
    shaders::{Material, Texture},
};
use crate::engine::rendering::shaders::ShaderType;
use glm::Matrix4;
use std::{cell::RefCell, rc::Rc};

pub struct GfxDevice {
    instance: Rc<dyn GfxApiDevice>,
    cmd_ids: u128,

    pub shader_api: Rc<dyn GfxApiShader>,
}

#[derive(Clone)]
pub struct ShaderModule {
    pub self_handle: u32,
    pub vertex_handle: Option<u32>,   // they can be deleted already
    pub fragment_handle: Option<u32>, // they can be deleted already
    pub texture_handles: Vec<u32>,    // Can be empty
    pub material: Material,
}

#[derive(Debug, Clone)]
pub struct BufferModule {
    pub handle: u32,
    pub buffer_handles: Option<Vec<u32>>,
    pub buffer_attributes: Option<Vec<f32>>,
    pub vertices: Option<Vec<Vec<f32>>>,
    pub vertices_count: Option<Vec<u32>>,
}

#[derive(Clone)]
pub struct RenderCommand {
    pub initialized: bool,
    pub handle: RenderCmdHd,
    pub shader_module: ShaderModule,
    pub buffer_module: BufferModule,
}

// ==============================
// sp_hdl - shader program handle
pub trait GfxApiShader {
    fn set_attribute_i32(&self, sp_hdl: u32, _identifier: &str, _value: i32);
    fn set_attribute_f32(&self, sp_hdl: u32, _identifier: &str, _value: f32);
    fn set_attribute_mat4(&self, sp_hdl: u32, _identifier: &str, _value: &Matrix4<f32>);
    fn set_attribute_bool(&self, sp_hdl: u32, _identifier: &str, _value: bool);
    fn set_attribute_color(&self, sp_hdl: u32, _identifier: &str, _value: glm::Vec4);
    fn set_texture_unit(&self, prog_hdl: u32, texture_pos: i32);
}

pub trait GfxApiDevice {
    // ======================
    // Shaders
    // ======================
    fn alloc_shader(&self, source: String, s_type: ShaderType) -> u32;
    fn alloc_shader_module(&self, vertex: u32, frag: u32, material: &Material) -> ShaderModule;
    fn release_shader_module(&self, module_handle: u32);
    fn use_shader_module(&self, module_handle: u32);

    // ======================
    // Buffers
    // ======================
    fn alloc_buffer(
        &self,
        vertices_set: Vec<Vec<f32>>,
        indices: Vec<Vec<u32>>,
        settings: BufferSettings,
    ) -> BufferModule;
    fn release_buffer(&self, module: BufferModule);
    fn alloc_framebuffer(&self, width: i32, height: i32) -> Result<FrameBuffer, &str>;
    fn use_framebuffer(&self, framebuffer: Option<&FrameBuffer>);
    fn blit_main_framebuffer(&self, buffer_module: &BufferModule, framebuffer: &FrameBuffer);

    // ======================
    // Textures
    // ======================
    fn alloc_framebuffer_texture(&self, width: i32, height: i32) -> u32;
    fn alloc_texture(&self, sp_hdl: u32, texture: &Texture) -> u32;
    fn release_texture(&self, tex_id: u32);

    // ======================
    // Drawing
    // ======================
    fn draw_command(&self, command: &RenderCommand);
    fn clear_color(&self);
    fn update_viewport(&self, x: u32, y: u32, width: u32, height: u32);
    fn set_update_viewport_callback(
        &self,
        window: &mut glfw::Window,
        viewport: RefCell<glm::Vector4<f32>>,
    );
    fn clear_buffers(&self);
}

impl GfxDevice {
    pub fn new(device_impl: Rc<dyn GfxApiDevice>, shader_impl: Rc<dyn GfxApiShader>) -> Self {
        Self {
            instance: device_impl,
            shader_api: shader_impl,
            cmd_ids: 0,
        }
    }

    pub fn use_framebuffer(&self, framebuffer: Option<&FrameBuffer>) {
        self.instance.use_framebuffer(framebuffer);
    }

    pub fn blit_main_framebuffer(&self, screen_module: &BufferModule, framebuffer: &FrameBuffer) {
        self.instance
            .blit_main_framebuffer(screen_module, framebuffer);
    }

    pub fn alloc_framebuffer(&self, width: i32, height: i32) -> FrameBuffer {
        self.instance
            .alloc_framebuffer(width, height)
            .expect(&format!(
                "[Gfx Device] Failed to allocate framebuffer (w: {}, h: {})",
                width, height
            ))
    }

    pub fn alloc_shader(&self, source: String, s_type: ShaderType) -> u32 {
        self.instance.alloc_shader(source, s_type)
    }

    pub fn alloc_shader_module(&self, vertex: u32, frag: u32, material: &Material) -> ShaderModule {
        self.instance.alloc_shader_module(vertex, frag, material)
    }

    pub fn use_shader_module(&self, module: &ShaderModule) {
        self.instance.use_shader_module(module.self_handle);
    }

    pub fn delete_shader_module(&self, module: ShaderModule) {
        self.instance.release_shader_module(module.self_handle);
    }

    pub fn alloc_texture(&self, sp_hdl: u32, texture: &Texture) -> u32 {
        self.instance.alloc_texture(sp_hdl, texture)
    }

    pub fn release_texture(&self, tex_id: u32) {
        self.instance.release_texture(tex_id);
    }

    // ======================
    // Buffers
    // ======================
    pub fn alloc_buffer(
        &self,
        vertices_set: Vec<Vec<f32>>,
        indices: Vec<Vec<u32>>,
        settings: BufferSettings,
    ) -> BufferModule {
        self.instance.alloc_buffer(vertices_set, indices, settings)
    }

    pub fn release_buffer(&self, module: BufferModule) {
        self.instance.release_buffer(module)
    }

    // ======================
    // Drawing
    // ======================
    pub fn build_command(
        &mut self,
        shad_mod: ShaderModule,
        buff_mod: BufferModule,
    ) -> RenderCommand {
        let id = self.cmd_ids;
        self.cmd_ids += 1;

        RenderCommand {
            handle: id,
            initialized: true,
            shader_module: shad_mod,
            buffer_module: buff_mod,
        }
    }

    pub fn draw_command(&self, command: &RenderCommand) {
        self.instance.draw_command(command)
    }

    pub fn update_viewport(&self, vp_rect: Rect<u32>) {
        self.instance
            .update_viewport(vp_rect.x, vp_rect.y, vp_rect.width, vp_rect.height);
    }

    pub fn set_update_viewport_callback(
        &self,
        window: &mut glfw::Window,
        viewport: RefCell<glm::Vector4<f32>>,
    ) {
        self.instance.set_update_viewport_callback(window, viewport);
    }

    pub fn clear(&self) {
        self.instance.clear_color();
        self.instance.clear_buffers();
    }
}
