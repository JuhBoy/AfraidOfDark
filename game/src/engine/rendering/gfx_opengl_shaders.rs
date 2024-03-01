use std::ffi::CString;

use super::gfx_device::GfxApiShader;

#[derive(Default)]
pub struct GfxOpenGLShaderApi;

impl GfxApiShader for GfxOpenGLShaderApi {
    fn set_attribute_i32(&self, prog_hdl: u32, identifier: &str, value: i32) {
        
        println!("Setting attribute: {} to {} [{} handle]", identifier, value, prog_hdl);

        unsafe { gl::UseProgram(prog_hdl); }

        match self.get_uniform_location(prog_hdl, identifier) {
            Ok(location) => {
                unsafe {
                    gl::Uniform1i(location, value);
                }
            }
            Err(e) => {
                println!("[Shader API Error]: {}", e);
            }
        }
        unsafe { gl::UseProgram(0); }
    }

    fn set_attribute_f32(&self, prog_hdl: u32, identifier: &str, value: f32) {
        match self.get_uniform_location(prog_hdl, identifier) {
            Ok(location) => {
                unsafe {
                    gl::Uniform1f(location, value);
                }
            }
            Err(e) => {
                println!("[Shader API Error]: {}", e);
            }
        }
    }

    fn set_attribute_bool(&self, prog_hdl: u32, identifier: &str, value: bool) {
        match self.get_uniform_location(prog_hdl, identifier) {
            Ok(location) => {
                unsafe {
                    gl::Uniform1i(location, if value == true { 1 } else { 0 });
                }
            }
            Err(e) => {
                println!("[Shader API Error]: {}", e);
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
                let err = format!("Uniform location not found: {}", identifier);
                return Err(err);
            }
            Ok(location)
        }
    }
}
