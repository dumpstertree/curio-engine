use cgmath::Matrix4;

use crate::collections::{quaternion::Quaternion, vector3::Vector3, vector4::Vector4};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Matrix4x4 {
    model: [[f32; 4]; 4],
}
impl Matrix4x4 {
    pub fn zero() -> Matrix4x4 {
        Matrix4x4::new(Vector3::zero(), Quaternion::zero(), Vector3::zero())
    }

    pub const fn default() -> Matrix4x4 {
        Matrix4x4 { model: [[0.0; 4]; 4] }
        // Matrix4x4::new(Vector3::zero(), Quaternion::identity(), Vector3::one())
    }

    fn transpose(m: &Matrix4x4) -> Matrix4x4 {
        let mut result = Matrix4x4 { model: [[0.0; 4]; 4] };
        for i in 0..4 {
            for j in 0..4 {
                result.model[i][j] = m.model[j][i];
            }
        }
        result
    }

    pub fn multiply_vec4(&self, v: Vector4) -> Vector4 {
        let m = &self.model;
        Vector4 {
            x: m[0][0] * v.x + m[0][1] * v.y + m[0][2] * v.z + m[0][3] * v.w,
            y: m[1][0] * v.x + m[1][1] * v.y + m[1][2] * v.z + m[1][3] * v.w,
            z: m[2][0] * v.x + m[2][1] * v.y + m[2][2] * v.z + m[2][3] * v.w,
            w: m[3][0] * v.x + m[3][1] * v.y + m[3][2] * v.z + m[3][3] * v.w,
        }
    }

    pub fn perspective_lh(fov_y: f32, aspect: f32, near: f32, far: f32) -> Matrix4x4 {
        let f = 1.0 / (fov_y * 0.5).tan();

        Matrix4x4::transpose(&Matrix4x4 {
            model: [
                [f / aspect, 0.0, 0.0, 0.0],
                [0.0, f, 0.0, 0.0],
                [0.0, 0.0, far / (far - near), 1.0],
                [0.0, 0.0, (-near * far) / (far - near), 0.0],
            ],
        })
    }

    pub fn look_at(eye: Vector3, target: Vector3, up: Vector3) -> Matrix4x4 {
        let forward = (target - eye).normalize_and_copy(); // camera's -Z
        let right = Vector3::cross(up, forward).normalize_and_copy(); // camera's +X
        let up_corrected = Vector3::cross(forward, right); // camera's +Y

        Matrix4x4::transpose(&Matrix4x4 {
            model: [
                [right.x, up_corrected.x, forward.x, 0.0],
                [right.y, up_corrected.y, forward.y, 0.0],
                [right.z, up_corrected.z, forward.z, 0.0],
                [
                    -Vector3::dot(right, eye),
                    -Vector3::dot(up_corrected, eye),
                    -Vector3::dot(forward, eye),
                    1.0,
                ],
            ],
        })
    }
    pub fn new(pos: Vector3, rot: Quaternion, scale: Vector3) -> Matrix4x4 {
        let pos2 = cgmath::Vector3::new(pos.x, pos.y, pos.z);
        let rot2 = cgmath::Quaternion::new(rot.w, rot.x, rot.y, rot.z);

        Matrix4x4 {
            model: (cgmath::Matrix4::from_translation(pos2)
                * cgmath::Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z)
                * cgmath::Matrix4::from(rot2))
            .into(),
        }
    }
    pub fn from_raw(matrix: [[f32; 4]; 4]) -> Matrix4x4 {
        Matrix4x4 { model: matrix }
    }
    pub fn from_cgmath(matrix: cgmath::Matrix4<f32>) -> Matrix4x4 {
        let mut m = [[0.0; 4]; 4];
        m[0][0] = matrix.x[0];
        m[0][1] = matrix.x[1];
        m[0][2] = matrix.x[2];
        m[0][3] = matrix.x[3];
        m[1][0] = matrix.y[0];
        m[1][1] = matrix.y[1];
        m[1][2] = matrix.y[2];
        m[1][3] = matrix.y[3];
        m[2][0] = matrix.z[0];
        m[2][1] = matrix.z[1];
        m[2][2] = matrix.z[2];
        m[2][3] = matrix.z[3];
        m[3][0] = matrix.w[0];
        m[3][1] = matrix.w[1];
        m[3][2] = matrix.w[2];
        m[3][3] = matrix.w[3];
        Matrix4x4 { model: m }
    }
    pub fn extract_rotation(&self) -> Quaternion {
        let forward = Vector3::new(
            self.model[2][0], // matrix.m02,
            self.model[2][1], // matrix.m12,
            self.model[2][2], // matrix.m22,
        );
        let upward = Vector3::new(
            self.model[1][0], // matrix.m01,
            self.model[1][1], // matrix.m11,
            self.model[1][2], // matrix.m21,
        );

        return Quaternion::from_look_rotation(forward, upward);
    }

    pub fn extract_position(self) -> Vector3 {
        let mut position = Vector3::zero();
        position.x = self.model[3][0]; // 3
        position.y = self.model[3][1]; // 12
        position.z = self.model[3][2]; // 23
        return position;
    }

    // public static Vector3 ExtractScale(this Matrix4x4 matrix)
    // {
    //     Vector3 scale;
    //     scale.x = new Vector4(matrix.m00, matrix.m10, matrix.m20, matrix.m30).magnitude;
    //     scale.y = new Vector4(matrix.m01, matrix.m11, matrix.m21, matrix.m31).magnitude;
    //     scale.z = new Vector4(matrix.m02, matrix.m12, matrix.m22, matrix.m32).magnitude;
    //     return scale;
    // }
}
impl Matrix4x4 {
    pub fn to_cg_math(&self) -> Matrix4<f32> {
        Matrix4::new(
            self.model[0][0],
            self.model[0][1],
            self.model[0][2],
            self.model[0][3],
            self.model[1][0],
            self.model[1][1],
            self.model[1][2],
            self.model[1][3],
            self.model[2][0],
            self.model[2][1],
            self.model[2][2],
            self.model[2][3],
            self.model[3][0],
            self.model[3][1],
            self.model[3][2],
            self.model[3][3],
        )
    }
    pub fn desc() -> egui_wgpu::wgpu::VertexBufferLayout<'static> {
        use std::mem;
        egui_wgpu::wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Matrix4x4>() as egui_wgpu::wgpu::BufferAddress,
            step_mode: egui_wgpu::wgpu::VertexStepMode::Instance,
            attributes: &[
                egui_wgpu::wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: egui_wgpu::wgpu::VertexFormat::Float32x4,
                },
                egui_wgpu::wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as egui_wgpu::wgpu::BufferAddress,
                    shader_location: 6,
                    format: egui_wgpu::wgpu::VertexFormat::Float32x4,
                },
                egui_wgpu::wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as egui_wgpu::wgpu::BufferAddress,
                    shader_location: 7,
                    format: egui_wgpu::wgpu::VertexFormat::Float32x4,
                },
                egui_wgpu::wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as egui_wgpu::wgpu::BufferAddress,
                    shader_location: 8,
                    format: egui_wgpu::wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
