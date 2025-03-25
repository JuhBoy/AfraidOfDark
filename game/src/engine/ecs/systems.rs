use super::{
    components::{Camera, SpriteRenderer2D, Transform},
    resources::RenderingFrameData,
};
use crate::engine::ecs::resources::CameraCullingState;
use crate::engine::utils::maths::Rect;
use bevy_ecs::query::Or;
use bevy_ecs::{
    entity::Entity,
    query::{Added, Changed},
    system::{Query, ResMut},
};

pub fn changed_sprite_2d_system(
    mut container: ResMut<RenderingFrameData>,
    mut cull_state: ResMut<CameraCullingState>,
    camera_query: Query<(&Camera, &Transform)>,
    mut sprites_query: Query<
        (Entity, &Transform, &SpriteRenderer2D),
        Or<(Changed<SpriteRenderer2D>, Changed<Transform>)>,
    >,
) {
    // Fetch camera data, width and height can't change for now, but position need a refresh.
    let (_, cam_tr) = camera_query.get(cull_state.camera_entity.unwrap()).unwrap();
    let camera_rect: Rect<f32> = Rect {
        x: cam_tr.position.x,
        y: cam_tr.position.y,
        ..cull_state.camera_world_viewport
    };

    for (entity, transform, _sprite_renderer_2d) in sprites_query.iter_mut() {
        container.updated_2d_render.push(entity);
        cull_state.update_visibility(entity, camera_rect, &transform);
    }
}

pub fn add_sprite_2d_system(
    mut container: ResMut<RenderingFrameData>,
    mut query: Query<(Entity, &Transform, &SpriteRenderer2D), Added<SpriteRenderer2D>>,
) {
    for (entity, _transform, sprite_renderer_2d) in query.iter_mut() {
        container.new_2d_render.push(entity);

        if let Some(tex) = sprite_renderer_2d.texture.as_ref() {
            println!("[Sprite 2D] Add with texture is {}", tex);
        } else {
            println!("[Sprite 2D] Add without texture");
        }
    }
}

pub fn add_camera_2d_system(
    mut container: ResMut<RenderingFrameData>,
    mut query: Query<(Entity, &Transform, &Camera), Added<Camera>>,
) {
    // todo! add multiple camera support
    if query.iter().count() > 1 {
        panic!("Only one camera is allowed in the scene");
    }

    for (entity, _transform, camera) in query.iter_mut() {
        println!(
            "[Camera Entity]: entity {} added camera: fov {} near {} far {}",
            entity.index(),
            camera.fov,
            camera.near,
            camera.far
        );
        container.updated_camera_transform.push(entity);
        container.updated_camera_settings.push(entity);
    }
}

/// Camera Update systems

pub fn update_camera_transform_system(
    mut container: ResMut<RenderingFrameData>,
    mut culling_state: ResMut<CameraCullingState>,
    query: Query<(Entity, &Camera), Changed<Transform>>,
) {
    culling_state.force_full_pass = true;
    query.iter().for_each(|(entity, _camera)| {
        container.updated_camera_transform.push(entity);
    });
}

pub fn update_camera_settings_system(
    mut container: ResMut<RenderingFrameData>,
    query: Query<Entity, Changed<Camera>>,
) {
    query.iter().for_each(|entity| {
        container.updated_camera_settings.push(entity);
    });
}

// End of camera Update systems
