use cgmath::Quaternion;

use crate::Collections::{matrix4x4, vector3::Vector3};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Matrix4x4 {
    model: [[f32; 4]; 4],
}
impl Matrix4x4 {
    pub fn zero() -> Matrix4x4 {
        Matrix4x4::new(Vector3::zero(), Quaternion::<f32>::new(0.0, 0.0, 0.0, 0.0), Vector3::zero())
    }
    pub fn default() -> Matrix4x4 {
        Matrix4x4::new(Vector3::zero(), Quaternion::<f32>::new(0.0, 0.0, 0.0, 0.0), Vector3::one())
    }
    pub fn new(pos: Vector3, rot: Quaternion<f32>, scale: Vector3) -> Matrix4x4 {
        Matrix4x4 {
            model: (cgmath::Matrix4::from_translation(pos.to_cg_math())
                * cgmath::Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z)
                * cgmath::Matrix4::from(rot))
            .into(),
        }
    }
}
impl Matrix4x4 {
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
