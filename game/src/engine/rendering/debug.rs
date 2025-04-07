use crate::engine::ecs::components::Transform;
use crate::engine::rendering::components::{RenderingCamera, ShaderStorageBuffer};
use crate::engine::rendering::gfx_device::{BufferModule, GfxDevice, RenderCommand};
use crate::engine::rendering::renderer_storage::RendererStorage;
use crate::engine::rendering::shaders::{Material, ShaderInfo, ShaderType};
use crate::engine::utils::maths::{compute_projection, compute_view_matrix, Grid, Rect};
use glm::Vector4;

const DEFAULT_GRID_WIDTH: i32 = 1000;
const DEFAULT_GRID_HEIGHT: i32 = 1000;
const DEFAULT_GRID_LENGTH: f32 = 2000f32;
const DEFAULT_GRID_THICKNESS: f32 = 1.1f32;

pub struct Debug {}

pub struct DebugGrid {
    pub lines: Vec<(RenderCommand, u32)>,
}

impl Debug {
    pub fn new() -> Self {
        Debug {}
    }

    pub fn build_grid(device: &mut GfxDevice, store: &RendererStorage) -> DebugGrid {
        // build the grid with the lowest vertex count possible
        let grid = Grid::with_minimal_points(
            DEFAULT_GRID_WIDTH,
            DEFAULT_GRID_HEIGHT,
            DEFAULT_GRID_LENGTH,
            DEFAULT_GRID_THICKNESS,
        );
        let mut grid_lines: Vec<(RenderCommand, u32)> = vec![];

        // load vertex shader
        let v_info = ShaderInfo::with_name(String::from("line_vertex.shader"), ShaderType::Vertex);
        let v_source = store.load_shader_content(&v_info);
        let v_shad = device.alloc_shader(v_source.unwrap(), ShaderType::Vertex);

        // load fragment shader
        let f_info =
            ShaderInfo::with_name(String::from("line_fragment.shader"), ShaderType::Fragment);
        let f_source = store.load_shader_content(&f_info);
        let f_shad = device.alloc_shader(f_source.unwrap(), ShaderType::Fragment);

        // full iteration columns and rows
        let cols_and_rows = grid.columns.iter().chain(grid.rows.iter());

        for (i, polyline) in cols_and_rows.enumerate() {
            let shader_module = device.alloc_shader_module(v_shad, f_shad, &Material::new());

            // Update the color for x:0 and y:0 row and column to show the center of the grid
            let mut color = Vector4::new(0f32, 0f32, 0f32, 0.4f32);
            if i == grid.columns.len() / 2 {
                color = Vector4::new(1f32, 0f32, 0f32, 1f32);
            } else if i == (grid.columns.len() + (grid.rows.len() / 2)) {
                color = Vector4::new(0f32, 1f32, 0f32, 1f32);
            }

            device
                .shader_api
                .set_attribute_f32(shader_module.self_handle, "thickness", 0.01f32);
            device.shader_api.set_attribute_color(
                shader_module.self_handle,
                "surface_color",
                color,
            );
            device.shader_api.set_attribute_vector2f(
                shader_module.self_handle,
                "offset",
                &glm::Vector2::new(
                    grid.columns.len() as f32 / 2f32,
                    grid.rows.len() as f32 / 2f32,
                ),
            );

            let points = polyline
                .points
                .iter()
                .map(|v| Vector4::new(v[0], v[1], v[2], 1f32))
                .collect();
            // Get raw data for one line
            let sso: ShaderStorageBuffer = device.alloc_shader_storage_buffer(&points);

            let render_cmd = device.build_command(
                shader_module,
                BufferModule {
                    handle: sso.vao_handle,
                    shader_storage: Option::from(sso),
                    buffer_handles: None,
                    buffer_attributes: None,
                    vertices: None,
                    vertices_count: None,
                },
            );

            // this is sent to the draw_gl_triangles opengl
            let vertex_count = (points.len() - 1) * 6;
            grid_lines.push((render_cmd, vertex_count as u32));
        }

        DebugGrid { lines: grid_lines }
    }
}

impl DebugGrid {
    pub fn draw(&self, device: &GfxDevice, camera: &RenderingCamera, rect: &Rect<u32>) {
        let transform = Transform::default();
        let trs_mat = &compute_view_matrix(&transform);

        let view_matrix = compute_view_matrix(&camera.transform);
        let proj_matrix = compute_projection(&camera, rect);

        for (cmd, vertices) in self.lines.iter() {
            // Update GPU data
            device
                .shader_api
                .set_attribute_mat4(cmd.shader_module.self_handle, "TRS", trs_mat);
            device.shader_api.set_attribute_mat4(
                cmd.shader_module.self_handle,
                "VIEW",
                &view_matrix,
            );
            device.shader_api.set_attribute_mat4(
                cmd.shader_module.self_handle,
                "PROJ",
                &proj_matrix,
            );

            // Draw the command
            device.draw_command(cmd, Option::from(*vertices as i32));
        }
    }
}
