// use egui_wgpu::wgpu::util::DeviceExt;

// #[repr(C)]
// #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
// struct QuadVertex {
//     pos: [f32; 2],
//     uv: [f32; 2],
// }

// impl QuadVertex {
//     fn desc<'a>() -> egui_wgpu::wgpu::VertexBufferLayout<'a> {
//         use std::mem;
//         egui_wgpu::wgpu::VertexBufferLayout {
//             array_stride: mem::size_of::<QuadVertex>() as egui_wgpu::wgpu::BufferAddress,
//             step_mode: egui_wgpu::wgpu::VertexStepMode::Vertex,
//             attributes: &[
//                 egui_wgpu::wgpu::VertexAttribute {
//                     offset: 0,
//                     shader_location: 0,
//                     format: egui_wgpu::wgpu::VertexFormat::Float32x2,
//                 },
//                 egui_wgpu::wgpu::VertexAttribute {
//                     offset: mem::size_of::<[f32; 2]>() as egui_wgpu::wgpu::BufferAddress,
//                     shader_location: 1,
//                     format: egui_wgpu::wgpu::VertexFormat::Float32x2,
//                 },
//             ],
//         }
//     }
// }

// fn create_quad(device: &egui_wgpu::wgpu::Device) -> egui_wgpu::wgpu::Buffer {
//     let vertices = [
//         QuadVertex { pos: [-1.0, -1.0], uv: [0.0, 0.0] },
//         QuadVertex { pos: [1.0, -1.0], uv: [1.0, 0.0] },
//         QuadVertex { pos: [1.0, 1.0], uv: [1.0, 1.0] },
//         QuadVertex { pos: [-1.0, 1.0], uv: [0.0, 1.0] },
//     ];
//     device.create_buffer_init(&egui_wgpu::wgpu::util::BufferInitDescriptor {
//         label: Some("Quad Vertex Buffer"),
//         contents: bytemuck::cast_slice(&vertices),
//         usage: egui_wgpu::wgpu::BufferUsages::VERTEX,
//     })
// }

// fn create_quad_indices(device: &egui_wgpu::wgpu::Device) -> egui_wgpu::wgpu::Buffer {
//     let indices: [u16; 6] = [0, 1, 2, 2, 3, 0];
//     device.create_buffer_init(&egui_wgpu::wgpu::util::BufferInitDescriptor {
//         label: Some("Quad Index Buffer"),
//         contents: bytemuck::cast_slice(&indices),
//         usage: egui_wgpu::wgpu::BufferUsages::INDEX,
//     })
// }
