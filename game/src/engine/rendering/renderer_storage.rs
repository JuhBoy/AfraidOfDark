use std::{
    collections::{HashMap, VecDeque}, rc::Rc
};

use crate::engine::utils::file_system::FileSystem;
use crate::engine::utils::file_system::FileType;

use super::{
    gfx_device::RenderCommand,
    renderer::{MeshInfo, RenderCmdHd}, shaders::ShaderInfo,
};

#[allow(dead_code)]
const SQUARE_NO_INDICES: [f32; 18] = [
    // FT
    -1.0, -1.0, 0.0,  // left 
     1.0, -1.0, 0.0,  // right
    -1.0,  1.0, 0.0,  // top 
    // ST
    -1.0,  1.0, 0.0,  // top
     1.0, -1.0, 0.0,  // right
     1.0,  1.0, 0.0   // bottom
];

pub struct RendererStorage {
    pub render_command_storage: HashMap<RenderCmdHd, Rc<RenderCommand>>,
    pub renderer_queue: VecDeque<Rc<RenderCommand>>,
}

impl RendererStorage {
    pub fn load(_mesh_info: &MeshInfo) -> Vec<Vec<f32>> {
        // unimplemented!("Load vertices not implemented yet");
        let mut vertices: Vec<Vec<f32>> = Vec::new();
        vertices.push(SQUARE_NO_INDICES.to_vec());

        for i in 0..vertices.len() {
            for j in 0..vertices[i].len() {
                vertices[i][j] = vertices[i][j] * 0.9;
            }
        }

        vertices
    }

    pub fn load_vertices(_file_path: &str) -> Vec<f32> {
        unimplemented!("Load vertices not implemented yet");
    }

    pub fn store_command(&mut self, cmd: RenderCommand, push_to_frame: bool) -> RenderCmdHd {
        let handle = cmd.handle;
        let rc_cmd: Rc<RenderCommand> = Rc::new(cmd);

        self.render_command_storage.insert(handle, rc_cmd.clone());

        if push_to_frame {
            self.renderer_queue.push_back(rc_cmd);
        }

        handle
    }

    pub fn get_ref(&self, hd: RenderCmdHd) -> Rc::<RenderCommand> {
        if let Some(x) = self.render_command_storage.get(&hd) {
            return x.clone();
        }
        panic!("Could not find Render Command Handle");
    }

    pub fn remove_render_command(&mut self, hd: RenderCmdHd) {
        self.render_command_storage.remove(&hd);
    }

    pub fn add_to_frame_queue(&mut self, hd: RenderCmdHd) {
        assert!(self.render_command_storage.contains_key(&hd));
        let cmd = self.render_command_storage.get(&hd).unwrap();
        self.renderer_queue.push_back(cmd.clone());
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
        println!("Loading shader: {}", &file_name);

        FileSystem::load_file(&file_name, FileType::Shader)
    }
}
