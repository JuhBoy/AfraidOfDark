use super::{
    components::MeshInfo, gfx_device::RenderCommand, renderer::RenderCmdHd, shaders::ShaderInfo,
};
use crate::engine::rendering::shaders::Texture;
use crate::engine::utils::file_system::FileSystem;
use crate::engine::utils::file_system::FileType;
use bit_set::BitSet;
use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
};

#[allow(dead_code)]
const SQUARE_NO_INDICES: [f32; 18] = [
    // FT
    -0.5, -0.5, 0.0, // left
    0.5, -0.5, 0.0, // right
    -0.5, 1.0, 0.0, // top
    // ST
    -0.5, 0.5, 0.0, // top
    0.5, -0.5, 0.0, // right
    0.5, 0.5, 0.0, // bottom
];

#[allow(dead_code)]
const SQUARE_WITH_UVS: [f32; 20] = [
    // positions            // texture coords
    0.5f32, 0.5f32, 0.0f32, 1.0f32, 1.0f32, // top right
    0.5f32, -0.5f32, 0.0f32, 1.0f32, 0.0f32, // bottom right
    -0.5f32, -0.5f32, 0.0f32, 0.0f32, 0.0f32, // bottom left
    -0.5f32, 0.5f32, 0.0f32, 0.0f32, 1.0f32, // top left
];

const SQUARE_2D_WITH_UVS: [f32; 24] = [
    // 2D positions		 // Tex coords
    -1.0f32, 1.0f32, 0.0f32, 1.0f32, -1.0f32, -1.0f32, 0.0f32, 0.0f32, 1.0f32, -1.0f32, 1.0f32,
    0.0f32, -1.0f32, 1.0f32, 0.0f32, 1.0f32, 1.0f32, -1.0f32, 1.0f32, 0.0f32, 1.0f32, 1.0f32,
    1.0f32, 1.0f32,
];

#[allow(dead_code)]
const INDICES: [u32; 6] = [0, 1, 3, 1, 2, 3];

struct HandleCountPair<T> {
    pub handle: T,
    pub count: u32,
}

pub struct RendererStorage {
    pub render_command_storage: HashMap<RenderCmdHd, Rc<RefCell<RenderCommand>>>,
    pub renderer_queue: RefCell<VecDeque<Rc<RefCell<RenderCommand>>>>,
    pub culled_handles: BitSet,

    ram_texture_cache: RefCell<HashMap<String, Rc<Texture>>>,
    gpu_texture_cache: HashMap<String, HandleCountPair<u32>>,
    dangling_textures: Vec<(String, u32)>,
}

impl RendererStorage {
    pub fn new() -> RendererStorage {
        RendererStorage {
            render_command_storage: HashMap::new(),
            renderer_queue: RefCell::new(VecDeque::new()),
            ram_texture_cache: RefCell::new(HashMap::new()),
            gpu_texture_cache: HashMap::new(),
            culled_handles: BitSet::with_capacity(2048),

            dangling_textures: Vec::with_capacity(200usize),
        }
    }

    pub fn load(_mesh_info: &MeshInfo) -> Vec<Vec<f32>> {
        // unimplemented!("Load vertices not implemented yet");
        let mut vertices: Vec<Vec<f32>> = Vec::new();
        vertices.push(SQUARE_WITH_UVS.to_vec());

        vertices
    }

    pub fn load_2d_quad() -> Vec<f32> {
        SQUARE_2D_WITH_UVS.to_vec()
    }

    pub fn get_quad_indices() -> Vec<u32> {
        INDICES.to_vec()
    }

    pub fn load_vertices(_file_path: &str) -> Vec<f32> {
        unimplemented!("Load vertices not implemented yet");
    }

    pub fn store_command(&mut self, cmd: RenderCommand, push_to_frame: bool) -> RenderCmdHd {
        let handle = cmd.handle;
        let cmd_ptr: Rc<RefCell<RenderCommand>> = Rc::new(RefCell::new(cmd));

        self.render_command_storage.insert(handle, cmd_ptr.clone());

        if push_to_frame {
            self.renderer_queue.borrow_mut().push_back(cmd_ptr);
        }

        handle
    }

    pub fn get_ref(&self, hd: RenderCmdHd) -> std::cell::Ref<'_, RenderCommand> {
        if let Some(x) = self.render_command_storage.get(&hd) {
            return x.borrow();
        }
        panic!("Could not find Render Command Handle");
    }

    pub fn get_mut_ref(&self, hd: RenderCmdHd) -> std::cell::RefMut<'_, RenderCommand> {
        if let Some(x) = self.render_command_storage.get(&hd) {
            return x.borrow_mut();
        }
        panic!("Could not find Render Command Handle");
    }

    pub fn remove_render_command(&mut self, hd: RenderCmdHd) {
        self.render_command_storage.remove(&hd);
    }

    pub fn add_to_frame_queue(&mut self, hd: RenderCmdHd) {
        debug_assert!(self.render_command_storage.contains_key(&hd));
        let cmd = self.render_command_storage.get(&hd).unwrap();
        self.renderer_queue.borrow_mut().push_back(cmd.clone());
    }

    pub fn load_shader_content(&self, shader_info: &ShaderInfo) -> Result<String, String> {
        let mut file_name: String = shader_info.file_name.clone();

        if file_name.contains("[[default]]") {
            file_name = match shader_info.shader_type {
                super::shaders::ShaderType::Fragment => String::from("fragment.shader"),
                super::shaders::ShaderType::Vertex => String::from("vertex.shader"),
            };
        }

        #[cfg(debug_assertions)]
        println!("[Shaders] Loading shader {}", &file_name);

        FileSystem::load_file(&file_name, FileType::Shader)
    }

    pub fn load_texture(&self, texture_name: &str) -> Result<Rc<Texture>, String> {
        let mut texture_cache = self.ram_texture_cache.borrow_mut();
        if texture_cache.contains_key(texture_name) {
            let texture = texture_cache[texture_name].clone();
            return Ok(texture);
        }

        let tex = FileSystem::load_texture(String::from(texture_name));

        if let Ok(texture) = tex {
            texture_cache.insert(String::from(texture_name), Rc::from(texture));
            return Ok(texture_cache[texture_name].clone());
        } else {
            println!("[Texture Loading]: failed for {}", texture_name);
        }

        Err("Unknown".to_owned())
    }

    pub fn mark_culled(&mut self, handle: RenderCmdHd, culled: bool) {
        if culled {
            self.culled_handles.insert(handle);
        } else {
            self.culled_handles.remove(handle);
        }
    }

    pub fn is_culled(&self, handle: RenderCmdHd) -> bool {
        self.culled_handles.contains(handle)
    }

    pub fn increment_texture_handle(&mut self, texture_name: &str, handle: u32) {
        if self.gpu_texture_cache.contains_key(texture_name) {
            self.gpu_texture_cache.get_mut(texture_name).unwrap().count += 1;
            return;
        }

        self.gpu_texture_cache.insert(
            String::from(texture_name),
            HandleCountPair { handle, count: 1 },
        );
    }

    pub fn decrement_texture_handle(&mut self, texture_name: &str) -> u32 {
        if self.gpu_texture_cache.contains_key(texture_name) {
            let texture_count_pair = self.gpu_texture_cache.get_mut(texture_name).unwrap();

            if texture_count_pair.count >= 1 {
                let count: u32 = texture_count_pair.count - 1;
                texture_count_pair.count = count;
                return count;
            }
        }

        0u32
    }

    pub fn has_gpu_texture_refs(&self, texture_name: &str) -> bool {
        self.gpu_texture_cache.contains_key(texture_name)
    }

    pub fn get_gpu_texture_handle(&self, texture_name: &str) -> u32 {
        self.gpu_texture_cache.get(texture_name).unwrap().handle
    }

    pub fn iter_dangling_textures<F>(&mut self, mut callback: F)
    where
        F: FnMut(&String, u32),
    {
        for (key, value) in self.gpu_texture_cache.iter() {
            if value.count == 0 {
                callback(key, value.handle);
                self.dangling_textures.push((key.clone(), value.handle));
            }
        }
    }

    pub fn reset_frame(&mut self) {
        for (tex_name, _hdl) in self.dangling_textures.iter() {
            self.gpu_texture_cache.remove(tex_name);
        }
        self.dangling_textures.clear();
    }

    pub fn log_all(&self) {
        let texture_count = self.gpu_texture_cache.len();
        let mut total_handles = 0u32;
        println!("Total texture in use: {}", texture_count);
        for (name, info) in self.gpu_texture_cache.iter() {
            println!(
                "\t[{}]: gpu handle: {} ref count: {}",
                name, info.handle, info.count
            );
            total_handles += info.count;
        }
        println!("Total handles: {}", total_handles);
        println!("-------");
    }
}
