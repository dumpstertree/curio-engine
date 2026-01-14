use core::panic;
use std::hash::Hash;

use egui_wgpu::wgpu::util::DeviceExt;
use egui_wgpu::wgpu::Buffer;
use egui_wgpu::wgpu::BufferAddress;
use egui_wgpu::wgpu::VertexAttribute;
use egui_wgpu::wgpu::VertexBufferLayout;
use mesh_tools::primitives::{generate_plane, generate_sphere};

use crate::collections::matrix4x4::Matrix4x4;
use crate::collections::vector3;
use crate::extensions::extensions_f32::ExtensionsF32;
use crate::random::Random;
use crate::system_adapters::adapter_system_gpu::SystemGPU;
#[derive(PartialEq)]
pub struct Mesh {
    pub instance_id: i32,
    pub name: String,
    pub verticies: Vec<Vertex>,
    pub indicies: Vec<u32>,
    pub matrix: Matrix4x4,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
}
impl Eq for Mesh {}
impl Hash for Mesh {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.instance_id.hash(state);
    }
}

impl Mesh {
    pub fn size(&self) -> crate::collections::vector3::Vector3 {
        let mut x_min: f32 = 0.0;
        let mut x_max: f32 = 0.0;
        let mut y_min: f32 = 0.0;
        let mut y_max: f32 = 0.0;
        let mut z_min: f32 = 0.0;
        let mut z_max: f32 = 0.0;

        for v in &self.verticies {
            if v.position[0] < x_min {
                x_min = v.position[0];
            }
            if v.position[0] > x_max {
                x_max = v.position[0];
            }
            if v.position[1] < y_min {
                y_min = v.position[1];
            }
            if v.position[1] > y_max {
                y_max = v.position[1];
            }
            if v.position[2] < z_min {
                z_min = v.position[2];
            }
            if v.position[2] > z_max {
                z_max = v.position[2];
            }
        }

        crate::collections::vector3::Vector3::new(x_max - x_min, y_max - y_min, z_max - z_min)
    }
    pub fn primitive_cube2(size: vector3::Vector3) -> Mesh {
        let (positions, indices, uvs, normals) = Mesh::create_cube(size);

        let mut v: Vec<Vertex> = Vec::new();
        for x in 0..positions.len() {
            let mut vertex = Vertex::default();
            vertex.position[0] = positions[x][0];
            vertex.position[1] = positions[x][1];
            vertex.position[2] = positions[x][2];
            vertex.uv0[0] = uvs[x][0];
            vertex.uv0[1] = uvs[x][1];
            vertex.normal[0] = normals[x][0];
            vertex.normal[1] = normals[x][1];
            vertex.normal[2] = normals[x][2];
            v.push(vertex);
        }
        let mut i: Vec<u32> = Vec::new();
        for x in indices {
            i.push(x as u32);
        }
        Mesh::new(String::from("Cube"), v, i, Matrix4x4::default())
    }
    pub fn create_cube(size: vector3::Vector3) -> (Vec<[f32; 3]>, Vec<i32>, Vec<[f32; 2]>, Vec<[f32; 3]>) {
        // Each face has 2 triangles = 6 vertices
        // Cube has 6 faces = 36 vertices total

        let positions: Vec<[f32; 3]> = vec![
            // 8 corners of the cube
            [size.x * -0.5, size.y * -0.5, size.z * 0.5],  // 0 front-bottom-left
            [size.x * 0.5, size.y * -0.5, size.z * 0.5],   // 1 front-bottom-right
            [size.x * 0.5, size.y * 0.5, size.z * 0.5],    // 2 front-top-right
            [size.x * -0.5, size.y * 0.5, size.z * 0.5],   // 3 front-top-left
            [size.x * -0.5, size.y * -0.5, size.z * -0.5], // 4 back-bottom-left
            [size.x * 0.5, size.y * -0.5, size.z * -0.5],  // 5 back-bottom-right
            [size.x * 0.5, size.y * 0.5, size.z * -0.5],   // 6 back-top-right
            [size.x * -0.5, size.y * 0.5, size.z * -0.5],  // 7 back-top-left
        ];

        let uvs: Vec<[f32; 2]> = vec![
            [0.0, 0.0], // 0
            [1.0, 0.0], // 1
            [1.0, 1.0], // 2
            [0.0, 1.0], // 3
            [0.0, 0.0], // 4
            [1.0, 0.0], // 5
            [1.0, 1.0], // 6
            [0.0, 1.0], // 7
        ];

        let normals: Vec<[f32; 3]> = vec![
            [0.0, 0.0, 1.0], // front
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, -1.0], // back
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
        ];

        // Each face has 2 triangles (6 indices)
        let indices: Vec<i32> = vec![
            // Front face
            0, 1, 2, 0, 2, 3, // Back face
            5, 4, 7, 5, 7, 6, // Left face
            4, 0, 3, 4, 3, 7, // Right face
            1, 5, 6, 1, 6, 2, // Top face
            3, 2, 6, 3, 6, 7, // Bottom face
            4, 5, 1, 4, 1, 0,
        ];

        (positions, indices, uvs, normals)
    }
    pub fn primitive_plane(width: f32, height: f32, width_segments: usize, height_segments: usize) -> Mesh {
        let (positions, indices, normals, uvs) = generate_plane(width, height, width_segments, height_segments);

        let mut v: Vec<Vertex> = Vec::new();
        for x in 0..positions.len() {
            let mut vertex = Vertex::default();
            vertex.position[0] = positions[x].x;
            vertex.position[1] = positions[x].y;
            vertex.position[2] = positions[x].z;
            vertex.uv0[0] = uvs[x].x;
            vertex.uv0[1] = uvs[x].y;
            vertex.normal[0] = normals[x].x;
            vertex.normal[1] = normals[x].y;
            vertex.normal[2] = normals[x].z;
            v.push(vertex);
        }
        let mut i: Vec<u32> = Vec::new();
        for x in indices {
            i.push(x.a);
            i.push(x.b);
            i.push(x.c);
        }
        Mesh::new(String::from("Plane"), v, i, Matrix4x4::default())
    }
    pub fn primitive_cube(size: crate::collections::vector3::Vector3) -> Mesh {
        let cube_length = size.z;
        let cube_height = size.y;
        let cube_width = size.x;
        let mut v: Vec<Vertex> = Vec::new();
        v.push(Vertex {
            uv0: [0.0, 1.0],
            uv1: [0.0, 0.0],
            position: [-cube_length * 0.5, -cube_width * 0.5, cube_height * 0.5],
            normal: [0.0, 0.0, 0.0],
            color: [0.0, 0.0, 0.0, 1.0],
        });
        v.push(Vertex {
            uv0: [0.0, 1.0],
            uv1: [0.0, 0.0],
            position: [cube_length * 0.5, -cube_width * 0.5, cube_height * 0.5],
            normal: [0.0, 0.0, 0.0],
            color: [0.0, 0.0, 0.0, 1.0],
        });
        v.push(Vertex {
            uv0: [0.0, 1.0],
            uv1: [0.0, 0.0],
            position: [cube_length * 0.5, -cube_width * 0.5, -cube_height * 0.5],
            normal: [0.0, 0.0, 0.0],
            color: [0.0, 0.0, 0.0, 1.0],
        });
        v.push(Vertex {
            uv0: [0.0, 1.0],
            uv1: [0.0, 0.0],
            position: [-cube_length * 0.5, -cube_width * 0.5, -cube_height * 0.5],
            normal: [0.0, 0.0, 0.0],
            color: [0.0, 0.0, 0.0, 1.0],
        });
        v.push(Vertex {
            uv0: [0.0, 1.0],
            uv1: [0.0, 0.0],
            position: [-cube_length * 0.5, cube_width * 0.5, cube_height * 0.5],
            normal: [0.0, 0.0, 0.0],
            color: [0.0, 0.0, 0.0, 1.0],
        });
        v.push(Vertex {
            uv0: [0.0, 1.0],
            uv1: [0.0, 0.0],
            position: [cube_length * 0.5, cube_width * 0.5, cube_height * 0.5],
            normal: [0.0, 0.0, 0.0],
            color: [0.0, 0.0, 0.0, 1.0],
        });
        v.push(Vertex {
            uv0: [0.0, 1.0],
            uv1: [0.0, 0.0],
            position: [cube_length * 0.5, cube_width * 0.5, -cube_height * 0.5],
            normal: [0.0, 0.0, 0.0],
            color: [0.0, 0.0, 0.0, 1.0],
        });
        v.push(Vertex {
            uv0: [0.0, 1.0],
            uv1: [0.0, 0.0],
            position: [-cube_length * 0.5, cube_width * 0.5, -cube_height * 0.5],
            normal: [0.0, 0.0, 0.0],
            color: [0.0, 0.0, 0.0, 1.0],
        });

        let i = vec![
            3,
            1,
            0,
            3,
            2,
            1,
            // Cube Left Side Triangles
            3 + 4 * 1,
            1 + 4 * 1,
            0 + 4 * 1,
            3 + 4 * 1,
            2 + 4 * 1,
            1 + 4 * 1,
            // Cube Front Side Triangles
            3 + 4 * 2,
            1 + 4 * 2,
            0 + 4 * 2,
            3 + 4 * 2,
            2 + 4 * 2,
            1 + 4 * 2,
            // Cube Back Side Triangles
            3 + 4 * 3,
            1 + 4 * 3,
            0 + 4 * 3,
            3 + 4 * 3,
            2 + 4 * 3,
            1 + 4 * 3,
            // Cube Rigth Side Triangles
            3 + 4 * 4,
            1 + 4 * 4,
            0 + 4 * 4,
            3 + 4 * 4,
            2 + 4 * 4,
            1 + 4 * 4,
            // Cube Top Side Triangles
            3 + 4 * 5,
            1 + 4 * 5,
            0 + 4 * 5,
            3 + 4 * 5,
            2 + 4 * 5,
            1 + 4 * 5,
        ];
        Mesh::new(String::from("Cube"), v, i, Matrix4x4::default())
    }
    pub fn primitive_sphere(diameter: f32, width_segments: usize, height_segments: usize) -> Mesh {
        let (positions, indices, normals, uvs) = generate_sphere(diameter / 2.0, width_segments, height_segments);

        let mut v: Vec<Vertex> = Vec::new();
        for x in 0..positions.len() {
            let mut vertex = Vertex::default();
            vertex.position[0] = positions[x].x;
            vertex.position[1] = positions[x].y;
            vertex.position[2] = positions[x].z;
            vertex.uv0[0] = uvs[x].x;
            vertex.uv0[1] = uvs[x].y;
            vertex.normal[0] = normals[x].x;
            vertex.normal[1] = normals[x].y;
            vertex.normal[2] = normals[x].z;
            v.push(vertex);
        }
        let mut i: Vec<u32> = Vec::new();
        for x in indices {
            i.push(x.a);
            i.push(x.b);
            i.push(x.c);
        }
        Mesh::new(String::from("Sphere"), v, i, Matrix4x4::default())
    }
    pub fn new(name: String, verticies: Vec<Vertex>, indicies: Vec<u32>, matrix: Matrix4x4) -> Mesh {
        let device = SystemGPU::get_device();
        let i_buffer = device.create_buffer_init(&egui_wgpu::wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indicies),
            usage: egui_wgpu::wgpu::BufferUsages::INDEX,
        });
        let v_buffer = device.create_buffer_init(&egui_wgpu::wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&verticies),
            usage: egui_wgpu::wgpu::BufferUsages::VERTEX,
        });
        Mesh {
            name,
            verticies,
            indicies,
            instance_id: Random::range_int(-9999, 9999),
            vertex_buffer: v_buffer,
            index_buffer: i_buffer,
            matrix,
        }
    }
    pub fn get_num_verticies(&self) -> i32 {
        self.verticies.len() as i32
    }

    pub fn get_vertex_buffer_for_device(&self) -> &Buffer {
        &self.vertex_buffer
    }
    pub fn get_index_buffer_for_device(&self) -> &Buffer {
        &self.index_buffer
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, serde::Serialize, serde::Deserialize, PartialEq)]

pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 4],
    pub uv0: [f32; 2],
    pub uv1: [f32; 2],
}
impl Hash for Vertex {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.position[0].hash(state);
        self.position[1].hash(state);
        self.position[2].hash(state);
        self.normal[0].hash(state);
        self.normal[1].hash(state);
        self.normal[2].hash(state);
        self.color[0].hash(state);
        self.color[1].hash(state);
        self.color[2].hash(state);
        self.color[3].hash(state);
        self.uv0[0].hash(state);
        self.uv0[1].hash(state);
        self.uv1[0].hash(state);
        self.uv1[1].hash(state);
    }
}
impl Eq for Vertex {}

impl Vertex {
    pub fn new(position: [f32; 3], normal: [f32; 3], color: [f32; 4], uv0: [f32; 2], uv1: [f32; 2]) -> Vertex {
        Vertex { position, normal, color, uv0, uv1 }
    }
    pub fn default() -> Vertex {
        Vertex {
            position: [0.0, 0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
            color: [0.0, 0.0, 0.0, 0.0],
            uv0: [0.0, 0.0],
            uv1: [0.0, 0.0],
        }
    }
    pub fn desc() -> VertexBufferLayout<'static> {
        use std::mem;
        egui_wgpu::wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as BufferAddress,
            step_mode: egui_wgpu::wgpu::VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    // position
                    offset: mem::size_of::<[f32; 0]>() as BufferAddress,
                    shader_location: 0,
                    format: egui_wgpu::wgpu::VertexFormat::Float32x3,
                },
                VertexAttribute {
                    // normal
                    offset: mem::size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 1,
                    format: egui_wgpu::wgpu::VertexFormat::Float32x3,
                },
                VertexAttribute {
                    // color
                    offset: mem::size_of::<[f32; 6]>() as BufferAddress,
                    shader_location: 2,
                    format: egui_wgpu::wgpu::VertexFormat::Float32x4,
                },
                VertexAttribute {
                    // uv0
                    offset: mem::size_of::<[f32; 10]>() as BufferAddress,
                    shader_location: 3,
                    format: egui_wgpu::wgpu::VertexFormat::Float32x2,
                },
                VertexAttribute {
                    // uv1
                    offset: mem::size_of::<[f32; 12]>() as BufferAddress,
                    shader_location: 4,
                    format: egui_wgpu::wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}
