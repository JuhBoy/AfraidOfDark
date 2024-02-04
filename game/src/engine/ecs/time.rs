use bevy_ecs::prelude::*;

#[derive(Resource, Default)]
pub struct Time {
    pub frames: f64,
    pub time: f64,
    pub delta_time: f32,
    pub fixed_delta_time: f32
}