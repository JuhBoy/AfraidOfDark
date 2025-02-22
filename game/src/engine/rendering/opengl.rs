use std::cell::RefCell;
use std::mem::size_of;
use std::ptr;
use gl::types::{GLsizei, GLsizeiptr};
use gfx_device::GfxApiDevice;
use crate::engine::rendering::gfx_device;
use crate::engine::rendering::gfx_device::{BufferModule, RenderCommand, ShaderModule};
use crate::engine::rendering::shaders::ShaderType;
use crate::engine::rendering::shaders::Material;

use super::shaders::ShaderPack;

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
                    println!("[Shader] Vertex handle created: {} \n{}", &shader_handle, &source);
                }
                ShaderType::Fragment => {
                    shader_handle = gl::CreateShader(gl::FRAGMENT_SHADER);
                    println!("[Shader] Fragment handle created: {} \n{}", &shader_handle, &source);
                }
            }

            let cp_src = source.clone();
            gl::ShaderSource(
                shader_handle,
                1,
                &cp_src.as_ptr().cast(),
                ptr::null(),
            );
            gl::CompileShader(shader_handle);

            let mut success: i32 = 0;
            gl::GetShaderiv(shader_handle, gl::COMPILE_STATUS, &mut success);
            if success == 0 { 
                let mut info_log: Vec<u8> = vec![0; 512];
                gl::GetShaderInfoLog(shader_handle, 512, ptr::null_mut(), info_log.as_mut_ptr().cast());
                println!("[Shader] Compilation Error: {}", String::from_utf8(info_log).unwrap());
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
            texture_hadles: vec![],
            material: material.clone()
        }
    }

    fn use_shader_module(&self, module_handle: u32) {
        unsafe { gl::UseProgram(module_handle); }
    }

    fn release_shader_module(&self, module_handle: u32) {
        unsafe { gl::DeleteProgram(module_handle) }
    }

    fn alloc_buffer(&self, vertices_set: Vec<Vec<f32>>, indices: Vec<Vec<u32>>, keep_vertices: Option<bool>) -> BufferModule {
        let mut vao_handle = 0u32;
        let mut vbo_handles: Vec<u32> = Vec::new();

        unsafe {
            gl::GenVertexArrays(1, ptr::addr_of_mut!(vao_handle));
            gl::BindVertexArray(vao_handle);

            for vertex_buffer in &vertices_set {
                let mut vbo_hd = 0;
                let buffer_size: usize = size_of::<f32>() * vertex_buffer.len();
                let vertex_stride: usize = size_of::<f32>() * 5;

                gl::GenBuffers(1, ptr::addr_of_mut!(vbo_hd));
                gl::BindBuffer(gl::ARRAY_BUFFER, vbo_hd);
                gl::BufferData(gl::ARRAY_BUFFER, buffer_size as GLsizeiptr, vertex_buffer.as_ptr().cast(), gl::STATIC_DRAW);

                // position attribute
                gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, vertex_stride as GLsizei, ptr::null());
                gl::EnableVertexAttribArray(0);

                // uvs attribute
                gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, vertex_stride as GLsizei, (3 * size_of::<f32>()) as *const _);
                gl::EnableVertexAttribArray(1);

                let mut ebo_hd = 0;
                gl::GenBuffers(1, ptr::addr_of_mut!(ebo_hd));
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo_hd);
                gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (indices[0].len() * size_of::<u32>()) as GLsizeiptr, indices[0].as_ptr().cast(), gl::DYNAMIC_DRAW);

                vbo_handles.push(vbo_hd as u32);

                gl::BindVertexArray(0);
            }
        }

        let buffers_sizes: Vec<u32> = vertices_set.iter().map(|x| x.len() as u32).collect();

        BufferModule {
            handle: vao_handle,
            buffer_handles: Option::from(vbo_handles),
            buffer_attributes: None,
            vertices: if keep_vertices.is_some() { Option::from(vertices_set) } else { None },
            vertices_count: Option::from(buffers_sizes)
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

    fn draw_command(&self, command: &RenderCommand) {
        unsafe {
            gl::BindVertexArray(command.buffer_module.handle);
            gl::PolygonMode(gl::FRONT, gl::FILL);
            gl::PolygonMode(gl::BACK, gl::LINE);

            command.shader_module.texture_hadles.iter().enumerate().for_each(|(i, x)| {
                gl::ActiveTexture(gl::TEXTURE0 + i as u32);
                gl::BindTexture(gl::TEXTURE_2D, *x);
            });

            for buffer in command.buffer_module.buffer_handles.as_ref().unwrap().iter() {
                gl::BindBuffer(gl::ARRAY_BUFFER, *buffer);
                gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
            }

            gl::BindVertexArray(0);
            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }

    fn clear_color(&self) {
        unsafe {
            gl::ClearColor(0f32, 0f32, 0f32, 1.0f32);
        }
    }

    fn update_viewport(&self, x: u32, y: u32, width: u32, height: u32) {
        unsafe {
            gl::Viewport(x as i32, y as i32, width as i32, height as i32);
        }
    }

    fn set_update_viewport_callback(&self, window: &mut glfw::Window, viewport: RefCell<glm::Vector4<f32>>) {
        window.set_size_callback(move |window: &mut glfw::Window, w: i32, h: i32| {
            let (scaled_width, scaled_height) = window.get_framebuffer_size();

            let vp_borrow = viewport.borrow();
            let final_x = scaled_width as f32 * vp_borrow.x;
            let final_y = scaled_height as f32 * vp_borrow.y;
            let final_w = scaled_width as f32 * vp_borrow.z;
            let final_h = scaled_height as f32 * vp_borrow.w;

            #[cfg(debug_assertions)] {
                let (w_factor, h_factor) = window.get_content_scale();
                println!("Window screen coords resized: {}x{} (scale factor {}x{})", w, h, w_factor, h_factor);
            }

            unsafe {
                gl::Viewport(final_x as i32, final_y as i32, final_w as i32, final_h as i32);
                gl::Viewport(final_x as i32, final_y as i32, final_w as i32, final_h as i32);
            }
        });
    }

    fn clear_buffers(&self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }
}