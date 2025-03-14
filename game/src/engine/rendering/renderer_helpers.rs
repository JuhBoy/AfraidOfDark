use super::{
    components::Rect,
    shaders::{Material, ShaderPack},
};
use crate::engine::ecs::components::SpriteRenderer2D;
use crate::engine::rendering::components::RenderRequest;
use crate::engine::rendering::gfx_device::RenderCommand;
use crate::engine::rendering::renderer::RenderCmdHd;
use crate::engine::rendering::renderer_storage::RendererStorage;
use crate::engine::rendering::shaders::{ShaderInfo, ShaderType};
use std::cell::RefMut;

pub type MaterialUpdateMask = u8;
pub const TEXTURE_MASK: u8 = 1 << 0;
pub const COLOR_MASK: u8 = 1 << 1;
pub const TRANSFORM_MASK: u8 = 1 << 2;

#[derive(Clone, Debug)]
pub struct TextureUpdateReq {
    pub handle: RenderCmdHd,
    pub input_texture_handle: Option<(String, u32)>,
}

pub fn get_shader_info_or_default(render_request: &RenderRequest) -> [ShaderInfo; 2] {
    let vert_default: ShaderInfo = ShaderInfo::default(ShaderType::Vertex);
    let frag_default: ShaderInfo = ShaderInfo::default(ShaderType::Fragment);

    let vert_info = render_request
        .material
        .shaders
        .vertex
        .as_ref()
        .map(|info| info.clone())
        .unwrap_or(vert_default);

    let frag_info = render_request
        .material
        .shaders
        .fragment
        .as_ref()
        .map(|info| info.clone())
        .unwrap_or(frag_default);

    [vert_info, frag_info]
}

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

    Material {
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
    }
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

    if updating_mat.main_texture.is_some()
        && updating_mat.main_texture != rendering_mat.main_texture
    {
        update_mask |= TEXTURE_MASK;
    }
    if rendering_mat.color != updating_mat.color {
        update_mask |= COLOR_MASK;
    }

    update_mask
}

pub fn shader_texture_update(store: &mut RendererStorage, request: TextureUpdateReq) {
    let prev_texture = store
        .get_ref(request.handle)
        .shader_module
        .material
        .main_texture
        .clone();

    // If there was a previous texture reduce ref count from the store 
    if let Some(previous_texture) = prev_texture {
        store.decrement_texture_handle(&previous_texture);
    }
    // Adds the input texture to store (either increments or adds to the ref count)
    if let Some((tex_name, handle)) = request.input_texture_handle.as_ref() {
        store.increment_texture_handle(&tex_name, *handle);
    }

    // Clear all previous texture handles and adds the input texture to the render command
    // This is done that way because multi texturing is not yet implemented
    let mut command: RefMut<RenderCommand> = store.get_mut_ref(request.handle);
    command.shader_module.texture_handles.clear();
    if let Some((tex_name, handle)) = &request.input_texture_handle {
        command.shader_module.texture_handles.push(*handle);
        command.shader_module.material.main_texture = Option::from(tex_name.clone());
    }
}
