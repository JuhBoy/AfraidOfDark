# Rendering example

This is a Small example of how the rendering will look.

* it will use gfx_device to access OpenGL or Vulkan layer
* From this class you will be able de allocate buffers & shader
* Union those, you can send a RenderCommand to the graphic device

it's a big WIP environment; shader compilation, textures, buffers optimization
and many other things are not yet implemented. It's more of a reminder of the current
progress.


    let gfx_device = self.gfx_device.as_ref().unwrap();
    gfx_device.clear();
    let fragment_shader_info = ShaderInfo {
        file_path: String::from("assets/fragment.shader"),
        load_type: OnDemand,
        shader_type: ShaderType::Fragment,
    };
    let vertex_shader_info = ShaderInfo {
        file_path: String::from("assets/vertex.shader"),
        load_type: OnDemand,
        shader_type: ShaderType::Vertex,
    };

    let v_sid: u32 = gfx_device.shader_from_file(&vertex_shader_info).unwrap();
    let f_sid: u32 = gfx_device.shader_from_file(&fragment_shader_info).unwrap();
    let shader_module: ShaderModule = gfx_device.new_shader_module(v_sid, f_sid);

    let vertices: Vec<f32> = vec![
        -0.5f32, -0.5f32, 0.0f32,
        0.5f32, -0.5f32, 0.0f32,
        0.0f32, 0.5f32, 0.0f32,
    ];

    let buffer_module: BufferModule = gfx_device.alloc_buffer(vec!(vertices), Option::from(true));
    let render_cmd = gfx_device.build_command(shader_module, buffer_module);
    
    // Inside your rendering loop call those methode
    gfx_device.use_shader_module(&render_cmd.shader_module);
    gfx_device.draw_command(&render_cmd);