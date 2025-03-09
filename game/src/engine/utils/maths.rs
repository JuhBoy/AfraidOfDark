use crate::engine::ecs::components::{Position, Scale, Transform};
use glm::{vec3, BaseFloat, Matrix4};

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
