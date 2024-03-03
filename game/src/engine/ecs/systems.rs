use bevy_ecs::{
    entity::Entity,
    query::{Added, Changed},
    system::{Query, ResMut},
};

use super::{
    components::{Camera, SpriteRenderer2D, Transform},
    time::RenderingResourcesContainer,
};

pub fn changed_sprite_2d_system(mut container: ResMut<RenderingResourcesContainer>,
                                mut query: Query<(Entity, &Transform, &SpriteRenderer2D), Changed<SpriteRenderer2D>>) {
    for (entity, transform, sprite_renderer_2d) in query.iter_mut() {
        println!(
            "[SpriteRenderer2D]: entity {} changes: {} {}",
            entity.index(),
            transform.position.x,
            sprite_renderer_2d.texture.as_ref().or(Some(&String::from("None"))).unwrap());

        container.updated_2d_render.push(entity);
    }
}

pub fn add_sprite_2d_system(mut container: ResMut<RenderingResourcesContainer>,
                            mut query: Query<(Entity, &Transform, &SpriteRenderer2D), Added<SpriteRenderer2D>>) {
    for (entity, _transform, sprite_renderer_2d) in query.iter_mut() {
        container.new_2d_render.push(entity);

        if let Some(tex) = sprite_renderer_2d.texture.clone() {
            println!("[Sprite 2D] Add with texture is {}", tex);
        } else {
            println!("[Sprite 2D] Add without texture");
        }
    }
}

pub fn add_camera_2d_system(mut query: Query<(Entity, &Transform, &Camera), Added<Camera>>) {

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
    }
}

pub fn update_camera_2d_system(query: Query<(Entity, &Transform, &Camera), Changed<Transform>>) {
    query.iter().for_each(|(entity, transform, camera)| {
        println!("[Camera] Update (id: {})", entity.index());
    });
}
