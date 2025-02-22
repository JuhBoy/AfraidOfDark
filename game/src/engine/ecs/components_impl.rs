use crate::engine::rendering::shaders::Material;

use super::components::SpriteRenderer2D;

impl SpriteRenderer2D {
    pub fn from(texture: String, preserve_aspect: bool) -> SpriteRenderer2D {
        SpriteRenderer2D {
            texture: Some(texture),
            material: Some(Material::new()),
            preserve_aspect,
        }
    }
}
