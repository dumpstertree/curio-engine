use core::{collections::camera_uniform::CameraUniform, system_adapters::adapter_system_gpu::SystemGPU};

use egui_wgpu::wgpu::{self, BindGroup, BindGroupLayout, Buffer, util::DeviceExt};

pub struct CameraRenderingComponents {
    pub camera_bind_group_layout: BindGroupLayout,
    pub camera_bind_group: BindGroup,
    pub camera_buffer: Buffer,
}
impl CameraRenderingComponents {
    pub fn new(camera_uniform: CameraUniform) -> CameraRenderingComponents {
        let device = SystemGPU::get_device();

        let camera_buffer = device.create_buffer_init(&egui_wgpu::wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: egui_wgpu::wgpu::BufferUsages::UNIFORM | egui_wgpu::wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout = device.create_bind_group_layout(&egui_wgpu::wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: egui_wgpu::wgpu::ShaderStages::VERTEX,
                ty: egui_wgpu::wgpu::BindingType::Buffer {
                    ty: egui_wgpu::wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        });

        let camera_bind_group = device.create_bind_group(&egui_wgpu::wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        CameraRenderingComponents {
            camera_bind_group,
            camera_bind_group_layout,
            // camera_uniform,
            camera_buffer,
        }
    }
}
