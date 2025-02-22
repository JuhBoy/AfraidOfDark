use crate::engine::ecs::components::SpriteRenderer2D;

use super::shaders::{Material, ShaderPack};

pub fn prepare_material(sprite: &SpriteRenderer2D, material: Option<&Material>) -> Material {
    let sprite_texture = sprite.texture.clone();

    if let Some(mat) = material {
        return Material {
            color: mat.color,
            render_priority: mat.render_priority,
            main_texture: sprite_texture,
            shaders: mat.shaders.clone(),
            pixel_per_unit: mat.pixel_per_unit,
        };
    }

    return Material {
        color: glm::Vector4 { x: 1f32, y: 1f32, z: 1f32, w: 1f32 },
        render_priority: 0,
        main_texture: None,
        shaders: ShaderPack { vertex: None, fragment: None },
        pixel_per_unit: 100,
    };
}
