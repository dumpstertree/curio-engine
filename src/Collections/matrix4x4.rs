use cgmath::Matrix4;

use crate::Collections::{quaternion::Quaternion, vector3::Vector3};

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

    pub fn new(pos: Vector3, rot: Quaternion, scale: Vector3) -> Matrix4x4 {
        Matrix4x4 {
            model: (cgmath::Matrix4::from_translation(pos.to_cg_math())
                * cgmath::Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z)
                * cgmath::Matrix4::from(rot.to_cg_math()))
            .into(),
        }
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

        return Quaternion::look_rotation(forward, upward);
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
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Matrix4x4>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
