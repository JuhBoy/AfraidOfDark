use std::{ffi::CString, ptr};

use super::{gfx_device::GfxApiShader, shaders::Texture};

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

		fn set_attribute_color(&self, prog_hdl: u32, identifier: &str, value: glm::Vec4) {

			unsafe { gl::UseProgram(prog_hdl); }

			match self.get_uniform_location(prog_hdl, identifier) {
				Ok(location) => {
					let vec: [f32; 4] = *value.as_array();
					unsafe { gl::Uniform4fv(location, 1, vec.as_ptr()); }
				},
				Err(e) => { 
					println!("[Shader API Error]: {}", e);
				}
			}
		}

    fn set_attribute_f32(&self, prog_hdl: u32, identifier: &str, value: f32) {

			unsafe { gl::UseProgram(prog_hdl); }

			match self.get_uniform_location(prog_hdl, identifier) {
					Ok(location) => {
							unsafe { gl::Uniform1f(location, value); }
					}
					Err(e) => {
							println!("[Shader API Error]: {}", e);
					}
			}
    }

    fn set_attribute_bool(&self, prog_hdl: u32, identifier: &str, value: bool) {

			  unsafe { gl::UseProgram(prog_hdl); }

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

    fn set_texture(&self, prog_hdl: u32, texture: Texture, texture_location: i32) -> u32 {
        unsafe {
            gl::UseProgram(prog_hdl);

            let mut tex_hdl: u32 = 0;
            gl::GenTextures(1, ptr::addr_of_mut!(tex_hdl) as *mut u32);
            gl::BindTexture(gl::TEXTURE_2D, tex_hdl);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            let internal_format = if texture.channels == 3 { gl::RGB as i32 } else { gl::RGBA as i32 };
            let pixel_format = if texture.channels == 3 { gl::RGB } else { gl::RGBA };
            gl::TexImage2D(gl::TEXTURE_2D, 0, internal_format, texture.width as i32, texture.height as i32, 0, pixel_format, gl::UNSIGNED_BYTE, texture.data.as_ptr().cast());

            let texture_location_name = format!("texture{}", texture_location);
            let location = self.get_uniform_location(prog_hdl, &texture_location_name);

            if let Ok(loc) = location {
                gl::Uniform1i(loc, texture_location);
            }

            gl::BindTexture(gl::TEXTURE_2D, 0);

            tex_hdl
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

                let err = format!("Uniform location not found: {} (Uniforms count found in shader {})", identifier, count);

                return Err(err);
            }
            Ok(location)
        }
    }
}
