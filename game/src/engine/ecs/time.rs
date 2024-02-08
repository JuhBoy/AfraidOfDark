use bevy_ecs::prelude::*;

#[derive(Resource, Default)]
pub struct Time {
    pub frames: f64,
    pub time: f64,
    pub delta_time: f32,
    pub fixed_delta_time: f32
}

#[derive(Resource, Default)]
pub struct RenderingResourcesContainer {
    pub frame: f64,
    pub new_2d_render: Vec<Entity>,
    pub updated_2d_render: Vec<Entity>,
    pub deleted_2d_render: Vec<Entity>
}