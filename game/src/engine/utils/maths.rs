use crate::engine::ecs::components::{Position, Scale, Transform};
use crate::engine::rendering::components::RenderingCamera;
use glm::{vec3, BaseFloat, Matrix4};
use std::ops::{Add, Div, Mul, Sub};

pub fn identity_mat4() -> Matrix4<f32> {
    let identity: Matrix4<f32> = glm::Matrix4::new(
        glm::vec4(1.0, 0.0, 0.0, 0.0),
        glm::vec4(0.0, 1.0, 0.0, 0.0),
        glm::vec4(0.0, 0.0, 1.0, 0.0),
        glm::vec4(0.0, 0.0, 0.0, 1.0),
    );

    identity
}

pub fn mat4_scale(mat4: &Matrix4<f32>, obj_scale: &Scale) -> Matrix4<f32> {
    let mut scale_mat4: Matrix4<f32> = identity_mat4();
    scale_mat4.c0.x = obj_scale.x;
    scale_mat4.c1.y = obj_scale.y;
    scale_mat4.c2.z = obj_scale.z;

    mat4.mul_m(&scale_mat4)
}

pub fn mat4_translate(mat4: &Matrix4<f32>, position: &Position) -> Matrix4<f32> {
    // copy all base matrix properties
    let mut translation = Matrix4::new(mat4.c0, mat4.c1, mat4.c2, mat4.c3);

    // makes translation projected into upper matrix coordinate system
    let pos: glm::Vector3<f32> = vec3(position.x, position.y, position.z);
    translation.c3.x = mat4.c0.x * pos.x + mat4.c0.y * pos.y + mat4.c0.z * pos.z + mat4.c3.x;
    translation.c3.y = mat4.c1.x * pos.x + mat4.c1.y * pos.y + mat4.c1.z * pos.z + mat4.c3.y;
    translation.c3.z = mat4.c2.x * pos.x + mat4.c2.y * pos.y + mat4.c2.z * pos.z + mat4.c3.z;

    translation
}

pub fn mat4_rotate(mat4: &Matrix4<f32>, angle_in_degrees: f32) -> Matrix4<f32> {
    let radian_angle: f32 = angle_in_degrees.to_radians();

    // The engine is probably going to stay 2D forever so only manage 2D rotation for now
    let mut mat4_rotate = identity_mat4();
    mat4_rotate.c0.x = radian_angle.cos();
    mat4_rotate.c0.y = radian_angle.sin();
    mat4_rotate.c1.x = -radian_angle.sin();
    mat4_rotate.c1.y = radian_angle.cos();

    mat4.mul_m(&mat4_rotate)
}

pub fn compute_trs(transform: &Transform) -> Matrix4<f32> {
    let mut trs_matrix: Matrix4<f32> = identity_mat4();

    trs_matrix = mat4_translate(&trs_matrix, &transform.position);
    trs_matrix = mat4_rotate(&trs_matrix, transform.rotation.z);
    trs_matrix = mat4_scale(&mut trs_matrix, &transform.scale);

    trs_matrix
}

pub fn compute_view_matrix(transform: &Transform) -> Matrix4<f32> {
    // Compute the camera view matrix (which gets it's translation vector negated to move other objects to the camera)
    let view_matrix: Matrix4<f32> = glm::Matrix4::new(
        glm::vec4(1.0, 0.0, 0.0, 0f32),
        glm::vec4(0.0, 1.0, 0.0, 0f32),
        glm::vec4(0.0, 0.0, 1.0, 0f32),
        glm::vec4(
            -transform.position.x,
            -transform.position.y,
            -transform.position.z,
            1.0,
        ),
    );

    view_matrix
}

pub fn compute_projection(camera: &RenderingCamera, window_rect: &Rect<u32>) -> Matrix4<f32> {
    let width = window_rect.width as f32 / camera.ppu as f32;
    let height = window_rect.height as f32 / camera.ppu as f32;
    let h_w = width * 0.5f32;
    let h_h = height * 0.5f32;
    let right = h_w;
    let left = -h_w;
    let top = h_h;
    let bot = -h_h;
    let near = camera.near;
    let far = camera.far;

    let mut orthographic_projection = identity_mat4();

    orthographic_projection.c0.x = 2f32 / (right - left);
    orthographic_projection.c1.y = 2f32 / (top - bot);
    orthographic_projection.c2.z = 2f32 / (far - near);
    orthographic_projection.c3.x = -(right + left) / (right - left);
    orthographic_projection.c3.y = -(top + bot) / (top - bot);
    orthographic_projection.c3.z = -(far + near) / (far - near);

    orthographic_projection
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Rect<T> {
    pub x: T,
    pub y: T,
    pub width: T,
    pub height: T,
}

impl<T> Rect<T>
where
    T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T> + Copy + From<f32>,
{
    pub fn from(transform: &Transform) -> Self {
        Self {
            x: T::from(transform.position.x),
            y: T::from(transform.position.y),
            width: T::from(transform.scale.x),
            height: T::from(transform.scale.y),
        }
    }

    pub fn min_x(&self) -> T {
        let half_width = self.width * T::from(0.5f32);
        let min_x = (self.x - half_width);

        min_x
    }

    pub fn max_x(&self) -> T {
        let half_width = self.width * T::from(0.5f32);
        let max_x = (self.x + half_width);
        max_x
    }

    pub fn min_y(&self) -> T {
        let half_height = self.height * T::from(0.5f32);
        let min_y = (self.y - half_height);
        min_y
    }

    pub fn max_y(&self) -> T {
        let half_height = self.height * T::from(0.5f32);
        let max_y = (self.y + half_height);
        max_y
    }
}

pub fn intersects(base_rect: Rect<f32>, rect: Rect<f32>) -> bool {
    let x_axis = rect.min_x() <= base_rect.max_x() && rect.max_x() >= base_rect.min_x();
    let y_axis = rect.min_y() <= base_rect.max_y() && rect.max_y() >= base_rect.min_y();
    x_axis && y_axis
}
