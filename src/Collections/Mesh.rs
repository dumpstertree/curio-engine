use wgpu::util::DeviceExt;
use wgpu::Buffer;
use wgpu::BufferAddress;
use wgpu::VertexAttribute;
use wgpu::VertexBufferLayout;

use crate::Collections::matrix4x4::Matrix4x4;
#[derive(Clone)]
pub struct Mesh {
    pub name: String,
    pub verticies: Vec<Vertex>,
    pub indicies: Vec<u32>,
}

impl Mesh {
    pub fn new(name: String, verticies: Vec<Vertex>, indicies: Vec<u32>) -> Mesh {
        Mesh { name, verticies, indicies }
    }
    pub fn get_num_verticies(&self) -> i32 {
        self.verticies.len() as i32
    }

    pub fn get_instance_buffer_for_device(&self, device: &wgpu::Device, transforms: &Vec<Matrix4x4>) -> Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&transforms),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }
    pub fn get_vertex_buffer_for_device(&self, device: &wgpu::Device) -> Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.verticies),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }
    pub fn get_index_buffer_for_device(&self, device: &wgpu::Device) -> Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&self.indicies),
            usage: wgpu::BufferUsages::INDEX,
        })
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub uv0: [f32; 2],
    pub uv1: [f32; 2],
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 4],
}

impl Vertex {
    pub fn default() -> Vertex {
        Vertex {
            uv0: [0.0, 0.0],
            uv1: [0.0, 0.0],
            position: [0.0, 0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
            color: [0.0, 0.0, 0.0, 0.0],
        }
    }
    pub fn desc() -> VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    // uv0
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                VertexAttribute {
                    // uv1
                    offset: mem::size_of::<[f32; 2]>() as BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                VertexAttribute {
                    // position
                    offset: mem::size_of::<[f32; 4]>() as BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                VertexAttribute {
                    // normal
                    offset: mem::size_of::<[f32; 7]>() as BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                VertexAttribute {
                    // color
                    offset: mem::size_of::<[f32; 10]>() as BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
