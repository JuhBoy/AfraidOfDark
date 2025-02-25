use super::shaders::Material;

#[derive(Debug)]
pub struct BufferSettings {
    pub keep_vertices: bool,
    pub vertex_size: i32,
    pub uvs_size: i32,
}

#[derive(Debug)]
pub struct FrameBuffer {
    pub self_handle: u32,
    pub texture_attachement: u32,
    pub width: i32,
    pub height: i32,
}

pub struct MeshInfo {
    pub file_path: Option<String>,
    pub count: u8,
    pub vertices_set: Option<Vec<Vec<f32>>>,
}

pub struct RenderRequest {
    pub mesh_info: MeshInfo,
    pub material: Material,
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
