use std::ffi::CString;

use super::gfx_device::GfxApiShader;

#[derive(Default)]
pub struct GfxOpenGLShaderApi;

impl GfxApiShader for GfxOpenGLShaderApi {
    fn set_attribute_i32(&self, prog_hdl: u32, identifier: &str, value: i32) {
        println!(
            "Setting attribute: {} to {} [{} handle]",
            identifier, value, prog_hdl
        );

        unsafe {
            gl::UseProgram(prog_hdl);
        }

        match self.get_uniform_location(prog_hdl, identifier) {
            Ok(location) => unsafe {
                gl::Uniform1i(location, value);
            },
            Err(e) => {
                println!("[Shader API Error]: {}", e);
            }
        }
        unsafe {
            gl::UseProgram(0);
        }
    }

    fn set_attribute_color(&self, prog_hdl: u32, identifier: &str, value: glm::Vec4) {
        unsafe {
            gl::UseProgram(prog_hdl);
        }

        match self.get_uniform_location(prog_hdl, identifier) {
            Ok(location) => {
                let vec: [f32; 4] = *value.as_array();
                unsafe {
                    gl::Uniform4fv(location, 1, vec.as_ptr());
                }
            }
            Err(e) => {
                println!("[Shader API Error]: {}", e);
            }
        }
    }

    fn set_attribute_f32(&self, prog_hdl: u32, identifier: &str, value: f32) {
        unsafe {
            gl::UseProgram(prog_hdl);
        }

        match self.get_uniform_location(prog_hdl, identifier) {
            Ok(location) => unsafe {
                gl::Uniform1f(location, value);
            },
            Err(e) => {
                println!("[Shader API Error]: {}", e);
            }
        }
    }

    fn set_attribute_bool(&self, prog_hdl: u32, identifier: &str, value: bool) {
        unsafe {
            gl::UseProgram(prog_hdl);
        }

        match self.get_uniform_location(prog_hdl, identifier) {
            Ok(location) => unsafe {
                gl::Uniform1i(location, if value == true { 1 } else { 0 });
            },
            Err(e) => {
                println!("[Shader API Error]: {}", e);
            }
        }
    }

    fn set_texture_unit(&self, prog_hdl: u32, texture_id: i32) {
        unsafe {
            gl::UseProgram(prog_hdl);
        }

        let location: Result<i32, String> =
            self.get_uniform_location(prog_hdl, &format!("texture{}", texture_id));

        match location {
            Ok(tex_location) => unsafe {
                gl::Uniform1i(tex_location, texture_id);
            },
            Err(err) => {
                println!("[OpenGl Shader]: Failed to get uniform location {}", &err)
            }
        }
    }
}

impl GfxOpenGLShaderApi {
    fn get_uniform_location(&self, prog_hdl: u32, identifier: &str) -> Result<i32, String> {
        let c_string = CString::new(identifier).unwrap();

        unsafe {
            let location: i32 = gl::GetUniformLocation(prog_hdl, c_string.as_ptr());
            if location == -1 {
                let mut count: i32 = 0;
                gl::GetProgramiv(prog_hdl, gl::ACTIVE_UNIFORMS, &mut count);

                let err = format!(
                    "Uniform location not found: {} (Uniforms count found in shader {})",
                    identifier, count
                );

                return Err(err);
            }
            Ok(location)
        }
    }
}
