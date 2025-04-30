use super::components::{ARGB8Color, BufferSettings, FrameBuffer, ShaderStorageBuffer};
use super::shaders::Texture;
use crate::engine::rendering::gfx_device;
use crate::engine::rendering::gfx_device::{BufferModule, RenderCommand, ShaderModule};
use crate::engine::rendering::shaders::Material;
use crate::engine::rendering::shaders::ShaderType;
use gfx_device::GfxApiDevice;
use gl::types::{GLsizei, GLsizeiptr};
use glm::Vector4;
use std::cell::RefCell;
use std::ffi::CString;
use std::mem::size_of;
use std::ptr;

#[derive(Default)]
pub struct GfxDeviceOpengl;

impl GfxApiDevice for GfxDeviceOpengl {
    fn alloc_shader(&self, source: String, s_type: ShaderType) -> u32 {
        #[allow(unused)]
        let mut shader_handle = 0u32;

        unsafe {
            match s_type {
                ShaderType::Vertex => {
                    shader_handle = gl::CreateShader(gl::VERTEX_SHADER);
                    println!(
                        "[Shader] Vertex handle created: {} \n{}",
                        &shader_handle, &source
                    );
                }
                ShaderType::Fragment => {
                    shader_handle = gl::CreateShader(gl::FRAGMENT_SHADER);
                    println!(
                        "[Shader] Fragment handle created: {} \n{}",
                        &shader_handle, &source
                    );
                }
            }

            let source_ref: &str = &source;
            let source_c: CString = CString::new(source_ref).unwrap();
            gl::ShaderSource(shader_handle, 1, &source_c.as_ptr().cast(), ptr::null());
            gl::CompileShader(shader_handle);

            let mut success: i32 = 0;
            gl::GetShaderiv(shader_handle, gl::COMPILE_STATUS, &mut success);
            if success == 0 {
                let mut info_log: Vec<u8> = vec![0; 1024];
                gl::GetShaderInfoLog(
                    shader_handle,
                    1024,
                    ptr::null_mut(),
                    info_log.as_mut_ptr().cast(),
                );
                println!(
                    "[Shader] Compilation Error: {}",
                    String::from_utf8(info_log).unwrap()
                );
            }
        }

        shader_handle
    }

    fn alloc_shader_module(&self, vertex: u32, frag: u32, material: &Material) -> ShaderModule {
        #[allow(unused)]
        let mut program_handle = 0u32;
        let delete_shader: bool = true; // @todo: create a config for this

        let mut frag_hd = Option::from(frag);
        let mut vert_hd = Option::from(vertex);

        unsafe {
            program_handle = gl::CreateProgram();
            gl::AttachShader(program_handle, vertex);
            gl::AttachShader(program_handle, frag);
            gl::LinkProgram(program_handle);

            if delete_shader {
                gl::DeleteShader(vertex);
                gl::DeleteShader(frag);
                frag_hd = None;
                vert_hd = None;
            }

            let mut linked = 0;
            gl::GetProgramiv(program_handle, gl::LINK_STATUS, &mut linked);
            if linked == 0 {
                println!("[OpenGL Shader Link] Shader program is not linked!");
            }
        }

        ShaderModule {
            self_handle: program_handle,
            fragment_handle: frag_hd,
            vertex_handle: vert_hd,
            texture_handles: vec![],
            material: material.clone(),
        }
    }

    fn release_shader_module(&self, module_handle: u32) {
        unsafe { gl::DeleteProgram(module_handle) }
    }

    fn use_shader_module(&self, module_handle: u32) {
        unsafe {
            gl::UseProgram(module_handle);
        }
    }

    fn alloc_shader_storage_buffer(&self, data: &Vec<Vector4<f32>>) -> ShaderStorageBuffer {
        let mut buffer_handle = 0u32;
        let mut vao_handle: u32 = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut vao_handle as *mut u32);
            gl::BindVertexArray(vao_handle);

            gl::GenBuffers(1, &mut buffer_handle as *mut u32);
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, buffer_handle);
            gl::BufferData(
                gl::SHADER_STORAGE_BUFFER,
                (size_of::<Vector4<f32>>() * data.len()) as GLsizeiptr,
                data.as_ptr().cast(),
                gl::STATIC_DRAW,
            );
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, buffer_handle);

            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        ShaderStorageBuffer {
            vao_handle,
            self_handle: buffer_handle,
            count: data.len(),
        }
    }

    fn alloc_buffer(
        &self,
        vertices_set: Vec<Vec<f32>>,
        indices: Vec<Vec<u32>>,
        settings: BufferSettings,
    ) -> BufferModule {
        let mut vao_handle = 0u32;
        let mut buffer_handles: Vec<u32> = Vec::new();

        unsafe {
            gl::GenVertexArrays(1, ptr::addr_of_mut!(vao_handle));
            gl::BindVertexArray(vao_handle);

            for (i, vertex_buffer) in vertices_set.iter().enumerate() {
                let mut vbo_handles: u32 = 0;
                let buffer_size: usize = size_of::<f32>() * vertex_buffer.len();
                let vertex_stride: usize =
                    size_of::<f32>() * (settings.vertex_size + settings.uvs_size) as usize;

                gl::GenBuffers(1, ptr::addr_of_mut!(vbo_handles));
                gl::BindBuffer(gl::ARRAY_BUFFER, vbo_handles);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    buffer_size as GLsizeiptr,
                    vertex_buffer.as_ptr().cast(),
                    gl::STATIC_DRAW,
                );

                // position attribute
                gl::VertexAttribPointer(
                    0,
                    settings.vertex_size,
                    gl::FLOAT,
                    gl::FALSE,
                    vertex_stride as GLsizei,
                    ptr::null(),
                );
                gl::EnableVertexAttribArray(0);

                // uvs attribute
                gl::VertexAttribPointer(
                    1,
                    settings.uvs_size,
                    gl::FLOAT,
                    gl::FALSE,
                    vertex_stride as GLsizei,
                    ((settings.vertex_size as usize) * size_of::<f32>()) as *const _,
                );
                gl::EnableVertexAttribArray(1);

                if indices.len() > i {
                    let mut ebo_handles: u32 = 0;

                    gl::GenBuffers(1, ptr::addr_of_mut!(ebo_handles));
                    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo_handles);
                    gl::BufferData(
                        gl::ELEMENT_ARRAY_BUFFER,
                        (indices[i].len() * size_of::<u32>()) as GLsizeiptr,
                        indices[i].as_ptr().cast(),
                        gl::DYNAMIC_DRAW,
                    );

                    buffer_handles.push(ebo_handles);
                }

                buffer_handles.push(vbo_handles);

                gl::BindVertexArray(0);
            }
        }

        let buffers_sizes: Vec<u32> = vertices_set
            .iter()
            .map(|x: &Vec<f32>| x.len() as u32)
            .collect();

        BufferModule {
            handle: vao_handle,
            shader_storage: None,
            buffer_handles: Option::from(buffer_handles),
            buffer_attributes: None,
            vertices: settings.keep_vertices.then_some(vertices_set),
            vertices_count: Option::from(buffers_sizes),
        }
    }

    fn release_buffer(&self, module: BufferModule) {
        unsafe {
            gl::DeleteVertexArrays(1, ptr::addr_of!(module.handle));

            if let Some(handles) = module.buffer_handles {
                for handle in handles {
                    gl::DeleteBuffers(1, ptr::addr_of!(handle));
                }
            }
        }
    }

    fn alloc_framebuffer(&self, width: i32, height: i32) -> Result<FrameBuffer, &str> {
        let mut fbo: u32 = 0;
        #[allow(unused)]
        let mut tex_hdl: u32 = 0;
        let mut rbo_handle: u32 = 0;

        unsafe {
            gl::GenFramebuffers(1, &mut fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);

            tex_hdl = self.alloc_framebuffer_texture(width, height);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                tex_hdl,
                0,
            );

            gl::GenRenderbuffers(1, &mut rbo_handle);
            gl::BindRenderbuffer(gl::RENDERBUFFER, rbo_handle);
            gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, width, height);
            gl::BindRenderbuffer(gl::FRAMEBUFFER, 0);

            // Attach the renderbuffer to the framebuffer
            gl::FramebufferRenderbuffer(
                gl::FRAMEBUFFER,
                gl::DEPTH_STENCIL_ATTACHMENT,
                gl::RENDERBUFFER,
                rbo_handle,
            );

            let fbo_state: u32 = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            if fbo_state != gl::FRAMEBUFFER_COMPLETE {
                println!("[GFX DEVICE] failed to allocate new frame buffer");
            }

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        Ok(FrameBuffer {
            self_handle: fbo,
            texture_attachment: tex_hdl,
            width,
            height,
        })
    }

    fn use_framebuffer(&self, framebuffer: Option<&FrameBuffer>) {
        let mut handle: u32 = 0;

        if let Some(fbo) = framebuffer {
            handle = fbo.self_handle;
        }

        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, handle);

            // @warning: maybe a shortcut here.. if the framebuffer is empty then we assume it's bliting the main framebuffer to the screen
            if framebuffer.is_none() {
                gl::Enable(gl::DEPTH_TEST);
            } else {
                gl::Disable(gl::DEPTH_TEST)
            }
        }
    }

    fn blit_main_framebuffer(&self, screen_module: &BufferModule, framebuffer: &FrameBuffer) {
        unsafe {
            gl::BindVertexArray(screen_module.handle);
            gl::BindTexture(gl::TEXTURE_2D, framebuffer.texture_attachment);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }
    }

    fn alloc_framebuffer_texture(&self, width: i32, height: i32) -> u32 {
        let mut texture_handle: u32 = 0;

        unsafe {
            gl::GenTextures(1, &mut texture_handle);
            gl::BindTexture(gl::TEXTURE_2D, texture_handle);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0i32,
                gl::RGB as i32,
                width,
                height,
                0i32,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                ptr::null(),
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        texture_handle
    }

    fn alloc_texture(&self, prog_hdl: u32, texture: &Texture) -> u32 {
        unsafe {
            gl::UseProgram(prog_hdl);

            let mut tex_hdl: u32 = 0;
            gl::GenTextures(1, &mut tex_hdl);
            gl::BindTexture(gl::TEXTURE_2D, tex_hdl);

            let error = gl::GetError();
            if error != gl::NO_ERROR {
                println!("OpenGL error: {}", error);
            }

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            let internal_format = texture.get_format() as i32;
            let pixel_format = texture.get_format();
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                internal_format,
                texture.width as i32,
                texture.height as i32,
                0,
                pixel_format,
                gl::UNSIGNED_BYTE,
                texture.data.as_ptr().cast(),
            );

            gl::BindTexture(gl::TEXTURE_2D, 0);

            tex_hdl
        }
    }

    fn release_texture(&self, texture_id: u32) {
        unsafe {
            gl::DeleteTextures(1, &texture_id);
        }
    }

    fn draw_command(&self, command: &RenderCommand, procedural: Option<i32>) {
        unsafe {
            gl::BindVertexArray(command.buffer_module.handle);
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            // if there is a shader buffer object, bind it !
            if let Some(sso) = command.buffer_module.shader_storage.as_ref() {
                gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, sso.self_handle);
            }

            command
                .shader_module
                .texture_handles
                .iter()
                .enumerate()
                .for_each(|(i, x)| {
                    gl::ActiveTexture(gl::TEXTURE0 + i as u32);
                    gl::BindTexture(gl::TEXTURE_2D, *x);
                });

            if procedural.is_none() {
                gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
            } else {
                gl::DrawArrays(gl::TRIANGLES, 0, procedural.unwrap());
            }

            gl::BindVertexArray(0);
            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }

    fn clear_color(&self, color: ARGB8Color) {
        let red: f32 = color.r as f32 / 255.0;
        let green: f32 = color.g as f32 / 255.0;
        let blue: f32 = color.b as f32 / 255.0;
        let alpha: f32 = 1f32;

        unsafe {
            gl::ClearColor(red, green, blue, alpha);
        }
    }

    fn update_viewport(&self, x: u32, y: u32, width: u32, height: u32) {
        unsafe {
            gl::Viewport(x as i32, y as i32, width as i32, height as i32);
        }
    }

    fn set_update_viewport_callback(
        &self,
        window: &mut glfw::Window,
        viewport: RefCell<Vector4<f32>>,
    ) {
        window.set_size_callback(move |window: &mut glfw::Window, w: i32, h: i32| {
            let (scaled_width, scaled_height) = window.get_framebuffer_size();

            let vp_borrow = viewport.borrow();
            let final_x = scaled_width as f32 * vp_borrow.x;
            let final_y = scaled_height as f32 * vp_borrow.y;
            let final_w = scaled_width as f32 * vp_borrow.z;
            let final_h = scaled_height as f32 * vp_borrow.w;

            #[cfg(debug_assertions)]
            {
                let (w_factor, h_factor) = window.get_content_scale();
                println!(
                    "Window screen coords resized: {}x{} (scale factor {}x{})",
                    w, h, w_factor, h_factor
                );
            }

            unsafe {
                gl::Viewport(
                    final_x as i32,
                    final_y as i32,
                    final_w as i32,
                    final_h as i32,
                );
            }
        });
    }

    fn clear_buffers(&self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    fn enable_blending(&self) {
        unsafe {
            gl::PolygonMode(gl::BACK, gl::LINE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
    }
}
