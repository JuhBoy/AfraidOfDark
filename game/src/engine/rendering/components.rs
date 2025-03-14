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
    pub ppu: u32,
    pub background_color: ARGB8Color,

    pub transform: Transform,
}

#[derive(Debug, Clone, Copy)]
pub struct RenderingUpdateState {
    pub camera_settings: bool,
    pub camera_transform: bool,
    // ...
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

#[derive(Clone, Copy, Debug)]
pub struct ARGB8Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
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

impl ARGB8Color {
    pub fn black() -> ARGB8Color {
        ARGB8Color {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }
    }

    pub fn white() -> ARGB8Color {
        ARGB8Color {
            r: 1u8,
            g: 1u8,
            b: 1u8,
            a: 255,
        }
    }
}

impl RenderingUpdateState { 
    pub fn reset(&mut self) { 
        self.camera_settings = false;
        self.camera_transform = false;
    }
}