use crate::engine::ecs::components::Transform;
use crate::engine::utils::maths::{intersects, Rect};
use bevy_ecs::prelude::*;
use std::cmp::PartialEq;

#[derive(Resource, Default)]
pub struct Time {
    pub frames: u64,
    pub time: f64,
    pub delta_time: f32,
    pub fixed_delta_time: f32,
}

#[derive(Resource, Default)]
pub struct RenderingFrameData {
    pub frame: f64,

    pub new_2d_render: Vec<Entity>,
    pub updated_2d_render: Vec<Entity>,
    pub deleted_2d_render: Vec<Entity>,

    pub updated_camera_transform: Vec<Entity>,
    pub updated_camera_settings: Vec<Entity>,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CulledState {
    Visible,
    Hidden,
    ByPass,
}

#[derive(Resource, Default)]
pub struct CameraCullingState {
    pub last_check_frame: f64,

    // this is done when the camera itself moved (Remove when QuadTree Implemented)
    pub camera_entity: Option<Entity>,
    pub camera_world_viewport: Rect<f32>,
    pub force_full_pass: bool,
    pub entities: Vec<(Entity, CulledState)>,
}

impl CameraCullingState {
    pub fn update_visibility(
        &mut self,
        entity: Entity,
        camera_rect: Rect<f32>,
        entity_transform: &Transform,
    ) {
        let frustum_state =
            CameraCullingState::compute_visibility(self, camera_rect, entity_transform);
        self.entities.push((entity, frustum_state));
    }

    pub fn compute_visibility(
        &self,
        camera_rect: Rect<f32>,
        entity_transform: &Transform,
    ) -> CulledState {
        // If no main camera entity has been register don't operate frustum computation
        if self.camera_entity.is_none() {
            return CulledState::Visible;
        }

        let sprite_rect = Rect::from(entity_transform);

        let frustum_state: CulledState = if intersects(camera_rect, sprite_rect) {
            CulledState::Visible
        } else {
            CulledState::Hidden
        };

        frustum_state
    }

    pub fn reset(&mut self) {
        self.force_full_pass = false;
        self.entities.clear();
    }
}

impl CulledState {
    // bypassed culled states are always visible
    pub fn is_visible(&self) -> bool {
        self.is_bypassed() || (*self == CulledState::Visible)
    }

    pub fn is_hidden(&self) -> bool {
        self.is_bypassed() || (*self == CulledState::Hidden)
    }

    pub fn is_bypassed(&self) -> bool {
        *self == CulledState::ByPass
    }
}
