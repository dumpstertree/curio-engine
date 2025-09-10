use core::{collections::camera_uniform::CameraUniform, system_adapters::adapter_system_gpu::SystemGPU};
use egui_wgpu::wgpu::{self, BindGroup, BindGroupLayout, Buffer};
use std::num::NonZeroU64;

pub struct CameraRenderingComponents {
    pub camera_bind_group_layout: BindGroupLayout,
    pub camera_bind_group: BindGroup,
    pub camera_buffer: Buffer,
    pub max_cameras: usize,
}

impl CameraRenderingComponents {
    pub fn new(max_cameras: usize) -> CameraRenderingComponents {
        let device = SystemGPU::get_device();

        // Per-WGPU requirement: uniform buffer dynamic offsets must be 256-byte aligned.
        let aligned_size: u64 = 256;
        let buffer_size = (max_cameras as u64) * aligned_size;

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Layout: uniform buffer with dynamic offset
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    // you can optionally set a min_binding_size here:
                    min_binding_size: NonZeroU64::new(aligned_size),
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        });

        // IMPORTANT: bind one element (aligned_size) as the binding `size`.
        // This allows dynamic offsets to select which element inside the larger buffer will be used.
        let per_element_size = NonZeroU64::new(aligned_size).unwrap();

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &camera_buffer,
                    offset: 0,
                    // Bind only one element (not the whole buffer). This lets dynamic offsets work.
                    size: Some(per_element_size),
                }),
            }],
            label: Some("camera_bind_group"),
        });

        CameraRenderingComponents {
            camera_bind_group_layout,
            camera_bind_group,
            camera_buffer,
            max_cameras,
        }
    }

    pub fn update(&self, i: usize, camera_uniform: &CameraUniform) {
        assert!(i < self.max_cameras);
        let queue = SystemGPU::get_queue();
        let aligned_size: u64 = 256;
        let offset = (i as u64) * aligned_size;
        queue.write_buffer(&self.camera_buffer, offset, bytemuck::cast_slice(&[*camera_uniform]));
    }

    pub fn bind(&self, pass: &mut wgpu::RenderPass, i: usize) {
        let aligned_size: u32 = 256;
        let offset = (i as u32) * aligned_size;
        pass.set_bind_group(1, &self.camera_bind_group, &[offset]);
    }
}
