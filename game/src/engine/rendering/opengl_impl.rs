use crate::engine::rendering::shaders::Texture;
use gl::types;

#[cfg(feature = "opengl")]
impl Texture {
    pub fn get_format(&self) -> types::GLenum {
        if self.channels == 3 {
            gl::RGB 
        } else {
            gl::RGBA
        }
    }
}
