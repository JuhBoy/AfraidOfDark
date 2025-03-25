use std::sync::{Arc, Mutex};

use bevy_ecs::{component::Component, entity::Entity, system::Resource};

use crate::engine::rendering::components::ARGB8Color;
use crate::engine::{
    inputs::keyboard::Keyboard,
    rendering::{renderer::RenderCmdHd, shaders::Material},
};

#[derive(Debug, Default)]
pub enum Projection {
    #[default]
    Orthographic,
    Perspective, // It will likely not be implemented
}

#[derive(Component, Debug, Default)]
pub struct Camera {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub ppu: u32,

    pub viewport: (f32, f32, f32, f32), // NDC
    pub mode: Projection,
    pub output_target: Option<u128>,
    pub background_color: Option<ARGB8Color>,
}

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
    pub material: Option<Material>,
    pub preserve_aspect: bool,
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RendererHandleComponent {
    pub handle: RenderCmdHd,
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct NoCulling;

#[derive(Resource, Debug)]
pub struct Inputs {
    pub keyboard: Arc<Mutex<Keyboard>>,
}

#[derive(Resource, Debug)]
pub struct CameraBinding {
    pub cameras: Vec<(Entity, u32)>, // The entity camera and its associated framebuffer object
}

// Impls ************************************************************

impl Scale {
    pub fn one() -> Self {
        Scale {
            x: 1.0f32,
            y: 1.0f32,
            z: 1.0f32,
        }
    }
}

impl Camera {
    pub fn default() -> Self {
        Camera {
            fov: 80.0,
            near: 0.1,
            far: 50.0,
            ppu: 380u32,
            viewport: (0.0, 0.0, 1.0, 1.0),
            mode: Projection::Orthographic,
            output_target: Option::None,
            background_color: Option::None,
        }
    }

    pub fn default_transform() -> Transform {
        Transform {
            position: Position {
                x: 0.0,
                y: 0.0,
                z: -1.0,
            },
            rotation: Default::default(),
            scale: Default::default(),
        }
    }
}
