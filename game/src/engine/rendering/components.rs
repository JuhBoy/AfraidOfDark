use super::{renderer::RenderCmdHd, shaders::Material};
use crate::engine::ecs::components::Transform;

#[derive(Debug)]
pub struct BufferSettings {
    pub keep_vertices: bool,
    pub vertex_size: i32,
    pub uvs_size: i32,
}

#[derive(Debug)]
pub struct FrameBuffer {
    pub self_handle: u32,
    pub texture_attachment: u32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug)]
pub struct RenderingCamera {
    pub near: f32,
    pub far: f32,
    pub background_color: [f32; 3],
    
    pub transform: Transform,
}

pub struct MeshInfo {
    pub file_path: Option<String>,
    pub count: u8,
    pub vertices_set: Option<Vec<Vec<f32>>>,
}

pub struct RenderRequest {
    pub mesh_info: MeshInfo,
    pub material: Material,
    pub transform: Transform,
}

pub struct RenderUpdate {
    pub render_cmd: RenderCmdHd,
    pub mesh_info: Option<MeshInfo>,
    pub material: Option<Material>,
    pub transform: Option<Transform>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum RenderState {
    Opened,
    Closed,
}

#[derive(Clone, Copy, Debug)]
pub struct Rect<T> {
    pub x: T,
    pub y: T,
    pub width: T,
    pub height: T,
}

impl BufferSettings {
    pub fn quad_default() -> BufferSettings {
        BufferSettings {
            keep_vertices: false,
            vertex_size: 3,
            uvs_size: 2,
        }
    }
}
