use bevy_ecs::component::Component;

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
