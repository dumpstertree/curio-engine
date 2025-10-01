use cgmath::Matrix4;
use serde::Deserialize;
use serde::Serialize;

use crate::collections::{quaternion::Quaternion, vector3::Vector3, vector4::Vector4};

#[repr(C)]
#[derive(Copy, Clone, Serialize, Deserialize, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Matrix4x4 {
    pub model: [[f32; 4]; 4],
}

impl Matrix4x4 {
    pub fn zero() -> Matrix4x4 {
        Matrix4x4::new(Vector3::zero(), Quaternion::zero(), Vector3::zero())
    }

    pub const fn default() -> Matrix4x4 {
        Matrix4x4 { model: [[0.0; 4]; 4] }
    }

    pub fn transpose(m: &Matrix4x4) -> Matrix4x4 {
        let mut result = Matrix4x4 { model: [[0.0; 4]; 4] };
        for i in 0..4 {
            for j in 0..4 {
                result.model[i][j] = m.model[j][i];
            }
        }
        result
    }

    /// Column-major matrix multiplication: result = a * b
    pub fn multiply(a: &Matrix4x4, b: &Matrix4x4) -> Matrix4x4 {
        let mut result = [[0.0; 4]; 4];
        for row in 0..4 {
            for col in 0..4 {
                result[row][col] = a.model[row][0] * b.model[0][col] + a.model[row][1] * b.model[1][col] + a.model[row][2] * b.model[2][col] + a.model[row][3] * b.model[3][col];
            }
        }
        Matrix4x4 { model: result }
    }

    pub fn orthographic_fit_scene(scene_min: Vector3, scene_max: Vector3, light_view: &Matrix4x4) -> Matrix4x4 {
        let corners = [
            Vector4::new(scene_min.x, scene_min.y, scene_min.z, 1.0),
            Vector4::new(scene_min.x, scene_min.y, scene_max.z, 1.0),
            Vector4::new(scene_min.x, scene_max.y, scene_min.z, 1.0),
            Vector4::new(scene_min.x, scene_max.y, scene_max.z, 1.0),
            Vector4::new(scene_max.x, scene_min.y, scene_min.z, 1.0),
            Vector4::new(scene_max.x, scene_min.y, scene_max.z, 1.0),
            Vector4::new(scene_max.x, scene_max.y, scene_min.z, 1.0),
            Vector4::new(scene_max.x, scene_max.y, scene_max.z, 1.0),
        ];

        let mut min = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Vector3::new(f32::MIN, f32::MIN, f32::MIN);

        for c in corners.iter() {
            let light_space = light_view.multiply_vec4(*c);
            let p = Vector3::new(light_space.x, light_space.y, light_space.z);

            min.x = min.x.min(p.x);
            min.y = min.y.min(p.y);
            min.z = min.z.min(p.z);
            max.x = max.x.max(p.x);
            max.y = max.y.max(p.y);
            max.z = max.z.max(p.z);
        }

        Matrix4x4::orthographic_lh(min.x, max.x, min.y, max.y, min.z, max.z)
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
        Matrix4x4 {
            model: [[f / aspect, 0.0, 0.0, 0.0], [0.0, f, 0.0, 0.0], [0.0, 0.0, far / (far - near), 1.0], [0.0, 0.0, (-near * far) / (far - near), 0.0]],
        }
    }

    /// Column-major left-handed LookAt
    pub fn look_at(eye: Vector3, target: Vector3, up: Vector3) -> Matrix4x4 {
        let f = (target - eye).normalize_and_copy();
        let s = Vector3::cross(up, f).normalize_and_copy();
        let u = Vector3::cross(f, s);

        Matrix4x4 {
            model: [[s.x, u.x, -f.x, 0.0], [s.y, u.y, -f.y, 0.0], [s.z, u.z, -f.z, 0.0], [-Vector3::dot(s, eye), -Vector3::dot(u, eye), Vector3::dot(f, eye), 1.0]],
        }
    }

    /// Column-major left-handed orthographic projection
    pub fn orthographic_lh(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Matrix4x4 {
        let r_l = right - left;
        let t_b = top - bottom;
        let f_n = far - near;

        Matrix4x4 {
            model: [[2.0 / r_l, 0.0, 0.0, 0.0], [0.0, 2.0 / t_b, 0.0, 0.0], [0.0, 0.0, 1.0 / f_n, 0.0], [-(right + left) / r_l, -(top + bottom) / t_b, -near / f_n, 1.0]],
        }
    }

    // pub fn new(pos: Vector3, rot: Quaternion, scale: Vector3) -> Matrix4x4 {
    //     let pos2 = cgmath::Vector3::new(pos.x, pos.y, pos.z);
    //     let rot2 = cgmath::Quaternion::new(rot.w, rot.x, rot.y, rot.z);

    //     // Matrix4x4 {
    //     //     model: (cgmath::Matrix4::from_translation(pos2) * cgmath::Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z) * cgmath::Matrix4::from(rot2)).into(),
    //     // }

    //     // Matrix4x4 {
    //     //     model: (cgmath::Matrix4::from(rot2) * cgmath::Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z) * cgmath::Matrix4::from_translation(pos2)).into(),
    //     // }
    //     Matrix4x4 {
    //         model: Matrix4x4::from_cgmath(cgmath::Matrix4::from_translation(pos2) * cgmath::Matrix4::from(rot2) * cgmath::Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z)).model,
    //     }
    // }
    // pub fn new(pos: Vector3, rot: Quaternion, scale: Vector3) -> Matrix4x4 {
    //     let pos2 = cgmath::Vector3::new(pos.x, pos.y, pos.z);
    //     let rot2 = cgmath::Quaternion::new(rot.w, rot.x, rot.y, rot.z);

    //     let mat = cgmath::Matrix4::from_translation(pos2) * cgmath::Matrix4::from(rot2) * cgmath::Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z);

    //     Matrix4x4::from_cgmath(mat)
    // }
    pub fn new(pos: Vector3, rot: Quaternion, scale: Vector3) -> Self {
        let translation = cgmath::Matrix4::from_translation(cgmath::Vector3::new(pos.x, pos.y, pos.z));
        let rotation = cgmath::Matrix4::from(cgmath::Quaternion::new(rot.w, rot.x, rot.y, rot.z));
        let scaling = cgmath::Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z);

        let mat = translation * rotation * scaling;

        // Convert to column-major layout for WGSL
        Self {
            model: [
                [mat.x[0], mat.x[1], mat.x[2], mat.x[3]], // column 0
                [mat.y[0], mat.y[1], mat.y[2], mat.y[3]], // column 1
                [mat.z[0], mat.z[1], mat.z[2], mat.z[3]], // column 2
                [mat.w[0], mat.w[1], mat.w[2], mat.w[3]], // column 3
            ],
        }
    }

    pub fn from_raw(matrix: [[f32; 4]; 4]) -> Matrix4x4 {
        Matrix4x4 { model: matrix }
    }
    pub fn from_cgmath(mat: cgmath::Matrix4<f32>) -> Self {
        // let cols = mat.as_ref(); // cgmath::Matrix4 implements AsRef<[f32; 16]>

        // cgmath stores matrices column-major internally already,
        // so we can just copy directly. But we'll make it explicit for clarity:
        // Self {
        //     model: [
        //         [cols[0], cols[1], cols[2], cols[3]],     // column 0
        //         [cols[4], cols[5], cols[6], cols[7]],     // column 1
        //         [cols[8], cols[9], cols[10], cols[11]],   // column 2
        //         [cols[12], cols[13], cols[14], cols[15]], // column 3
        //     ],
        // }
        Self {
            model: [
                [mat.x[0], mat.x[1], mat.x[2], mat.x[3]], // column 0
                [mat.y[0], mat.y[1], mat.y[2], mat.y[3]], // column 1
                [mat.z[0], mat.z[1], mat.z[2], mat.z[3]], // column 2
                [mat.w[0], mat.w[1], mat.w[2], mat.w[3]], // column 3
            ],
        }
    }

    pub fn extract_rotation(&self) -> Quaternion {
        let forward = Vector3::new(self.model[2][0], self.model[2][1], self.model[2][2]);
        let upward = Vector3::new(self.model[1][0], self.model[1][1], self.model[1][2]);
        Quaternion::from_look_rotation(forward, upward)
    }

    pub fn extract_position(self) -> Vector3 {
        Vector3::new(self.model[3][0], self.model[3][1], self.model[3][2])
    }
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

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct QuadVertex {
    pos: [f32; 2],
    uv: [f32; 2],
}
