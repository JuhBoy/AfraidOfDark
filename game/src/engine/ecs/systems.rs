use bevy_ecs::{
    entity::Entity,
    query::{Added, Changed},
    system::{Query, ResMut},
};

use super::{
    components::{SpriteRenderer2D, Transform},
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
            println!("Sprite render 2d texture is {}", tex);
        } else {
            println!("Sprite render 2d texture is None");
        }
    }
}
