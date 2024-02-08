use std::mem::size_of;
use std::ptr;
use gl::types::{GLsizei, GLsizeiptr};
use gfx_device::GfxApiDevice;
use crate::engine::rendering::gfx_device;
use crate::engine::rendering::gfx_device::{BufferModule, RenderCommand, ShaderModule};
use crate::engine::rendering::shaders::ShaderType;

#[derive(Default)]
pub struct GfxDeviceOpengl;

impl GfxApiDevice for GfxDeviceOpengl {
    fn alloc_shader(&self, source: String, s_type: ShaderType) -> u32 {
        let mut shader_handle = 0u32;

        unsafe {
            match s_type {
                ShaderType::Vertex => {
                    shader_handle = gl::CreateShader(gl::VERTEX_SHADER);
                    println!("Vertex Shader handle created: {} \n{}", &shader_handle, &source);
                }
                ShaderType::Fragment => {
                    shader_handle = gl::CreateShader(gl::FRAGMENT_SHADER);
                    println!("Fragment Shader handle created: {} \n{}", &shader_handle, &source);
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
        }

        shader_handle
    }

    fn alloc_shader_module(&self, vertex: u32, frag: u32, delete_shader: Option<bool>) -> ShaderModule {
        let mut program_handle = 0u32;
        let delete_shader = delete_shader.unwrap_or(false);

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
        }

        ShaderModule {
            self_handle: program_handle,
            fragment_handle: frag_hd,
            vertex_handle: vert_hd,
        }
    }

    fn use_shader_module(&self, module_handle: u32) {
        unsafe { gl::UseProgram(module_handle); }
    }

    fn release_shader_module(&self, module_handle: u32) {
        unsafe { gl::DeleteProgram(module_handle) }
    }

    fn alloc_buffer(&self, vertices_set: Vec<Vec<f32>>, keep_vertices: Option<bool>) -> BufferModule {
        let mut vao_handle = 0u32;
        let mut vbo_handles: Vec<u32> = Vec::new();

        unsafe {
            gl::GenVertexArrays(0, ptr::addr_of_mut!(vao_handle));
            gl::BindVertexArray(vao_handle);

            for vertex_buffer in &vertices_set {
                let mut current_vbo_hd = 0;
                let buffer_size: usize = size_of::<f32>() * vertex_buffer.len();
                let vertex_stride: usize = size_of::<f32>() * 3;

                gl::GenBuffers(1, ptr::addr_of_mut!(current_vbo_hd));
                gl::BindBuffer(gl::ARRAY_BUFFER, current_vbo_hd);
                gl::BufferData(gl::ARRAY_BUFFER, buffer_size as GLsizeiptr, vertex_buffer.as_ptr().cast(), gl::STATIC_DRAW);

                gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, vertex_stride as GLsizei, ptr::null());
                gl::EnableVertexAttribArray(0);

                vbo_handles.push(current_vbo_hd as u32);
            }
        }

        BufferModule {
            module_handle: vao_handle,
            buffer_handles: Option::from(vbo_handles),
            buffer_attributes: None,
            vertices: if keep_vertices.is_some() { Option::from(vertices_set) } else { None },
        }
    }

    fn release_buffer(&self, module: BufferModule) {
        unsafe {
            gl::DeleteVertexArrays(1, ptr::addr_of!(module.module_handle));

            if let Some(handles) = module.buffer_handles {
                for handle in handles {
                    gl::DeleteBuffers(1, ptr::addr_of!(handle));
                }
            }
        }
    }

    fn draw_command(&self, command: &RenderCommand) {
        unsafe {
            gl::BindVertexArray(command.buffer_module.module_handle);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::BindVertexArray(0);
        }
    }

    fn clear_color(&self) {
        unsafe {
            gl::ClearColor(0f32, 0f32, 0f32, 1.0f32);
        }
    }

    fn clear_buffers(&self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }
}