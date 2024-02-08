use std::sync::Arc;

use bevy_ecs::component::Component;

use crate::engine::rendering::{renderer::RenderCmdHd, shaders::Material};

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Rotation {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Scale {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Transform {
    pub position: Position,
    pub rotation: Rotation,
    pub scale: Scale,
}

#[derive(Component, Debug, Default)]
pub struct SpriteRenderer2D {
    pub texture: Option<String>,
    pub material: Option<Arc<Material>>
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RendererHandleComponent {
    pub handle: RenderCmdHd
}