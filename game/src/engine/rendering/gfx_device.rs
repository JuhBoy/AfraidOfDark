use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use crate::engine::rendering::shaders::{ShaderInfo, ShaderType};

pub struct GfxDevice {
    instance: Rc<dyn GfxApiDevice>,
}

pub struct ShaderModule {
    pub self_handle: u32,
    pub vertex_handle: Option<u32>, // they can be deleted already
    pub fragment_handle: Option<u32>, // they can be deleted already
}

pub struct BufferModule {
    pub module_handle: u32,
    pub buffer_handles: Option<Vec<u32>>,
    pub buffer_attributes: Option<Vec<f32>>,
    pub vertices: Option<Vec<Vec<f32>>>
}

pub struct RenderCommand {
    pub initialized: bool,
    pub shader_module: ShaderModule,
    pub buffer_module: BufferModule
}

pub struct Vbo;

pub trait GfxApiDevice {

    // ======================
    // Shaders
    // ======================
    fn alloc_shader(&self, source: String, s_type: ShaderType) -> u32;
    fn alloc_shader_module(&self, vertex: u32, frag: u32, delete_shader: Option<bool>) -> ShaderModule;
    fn release_shader_module(&self, module_handle: u32);
    fn use_shader_module(&self, module_handle: u32);

    // ======================
    // Buffers
    // ======================
    fn alloc_buffer(&self, vertices_set: Vec<Vec<f32>>, keep_vertices: Option<bool>) -> BufferModule;
    fn release_buffer(&self, module: BufferModule);

    // ======================
    // Drawing
    // ======================
    fn draw_command(&self, command: &RenderCommand);
    fn clear_color(&self);
    fn clear_buffers(&self);
}

impl GfxDevice {
    pub fn new(device_impl: Rc<dyn GfxApiDevice>) -> Self {
        Self {
            instance: device_impl
        }
    }

    pub fn shader_from_file(&self, shader_info: &ShaderInfo) -> Result<u32, String> {
        return match File::open(&shader_info.file_path).as_mut() {
            Ok(file) => {
                let mut file_content: String = String::new();
                file.read_to_string(&mut file_content).expect("Failed to load shader");
                Ok(self.instance.alloc_shader(file_content, shader_info.shader_type))
            }
            Err(_) => Err(String::from("Failed to load shader content"))
        }
    }

    pub fn new_shader_module(&self, vertex: u32, frag: u32) -> ShaderModule {
        self.instance.alloc_shader_module(vertex, frag, Option::from(true))
    }

    pub fn use_shader_module(&self, module: &ShaderModule) {
        self.instance.use_shader_module(module.self_handle);
    }

    pub fn delete_shader_module(&self, module: ShaderModule) {
        self.instance.release_shader_module(module.self_handle);
    }

    // ======================
    // Buffers
    // ======================
    pub fn alloc_buffer(&self, vertices_set: Vec<Vec<f32>>, keep_vertices: Option<bool>) -> BufferModule {
        self.instance.alloc_buffer(vertices_set, keep_vertices)
    }

    pub fn release_buffer(&self, module: BufferModule) {
        self.instance.release_buffer(module)
    }

    // ======================
    // Drawing
    // ======================
    pub fn build_command(&self, shad_mod: ShaderModule, buff_mod: BufferModule) -> RenderCommand {
        RenderCommand {
            initialized: true,
            shader_module: shad_mod,
            buffer_module: buff_mod
        }
    }

    pub fn draw_command(&self, command: &RenderCommand) {
        self.instance.draw_command(command)
    }
    pub fn clear(&self) {
        self.instance.clear_color();
        self.instance.clear_buffers();
    }
}