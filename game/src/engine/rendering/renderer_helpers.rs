use crate::engine::ecs::components::SpriteRenderer2D;

use super::{
    components::Rect,
    shaders::{Material, ShaderPack},
};

pub type MaterialUpdateMask = u8;
pub const TEXTURE_MASK: u8 = 1 << 0;
pub const COLOR_MASK: u8 = 1 << 1;

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
        color: glm::Vector4 {
            x: 1f32,
            y: 1f32,
            z: 1f32,
            w: 1f32,
        },
        render_priority: 0,
        main_texture: None,
        shaders: ShaderPack {
            vertex: None,
            fragment: None,
        },
        pixel_per_unit: 100,
    };
}

pub fn compute_gfx_viewport_rect(viewport: &glm::Vector4<f32>, window: &glfw::Window) -> Rect<u32> {
    let (scaled_width, scaled_height) = window.get_framebuffer_size();

    let final_x: f32 = scaled_width as f32 * viewport.x;
    let final_y: f32 = scaled_height as f32 * viewport.y;
    let final_w: f32 = scaled_width as f32 * viewport.z;
    let final_h: f32 = scaled_height as f32 * viewport.w;

    Rect {
        x: final_x as u32,
        y: final_y as u32,
        width: final_w as u32,
        height: final_h as u32,
    }
}

pub fn get_material_changes(
    rendering_mat: &Material,
    updating_mat: &Material,
) -> MaterialUpdateMask {
    let mut update_mask: u8 = 0;

    if updating_mat.main_texture.is_some() {
        update_mask |= TEXTURE_MASK;
    }
    if rendering_mat.color != updating_mat.color {
        update_mask |= COLOR_MASK;
    }

    return update_mask;
}
