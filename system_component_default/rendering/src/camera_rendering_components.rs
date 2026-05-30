use camera::camera_uniform::CameraUniform;
use curio_core::engine_services::services;
use egui_wgpu::wgpu::{self, BindGroup, BindGroupLayout, Buffer};
use std::num::NonZeroU64;

/// Stores GPU-side bindings for camera data (projection, view, tint, etc.)
pub struct CameraRenderingComponents {
    pub camera_bind_group_layout: BindGroupLayout,
    pub camera_bind_group: BindGroup,
    pub camera_buffer: Buffer,
    pub max_cameras: usize,
}

impl CameraRenderingComponents {
    pub fn new(max_cameras: usize) -> CameraRenderingComponents {
        assert!(max_cameras > 0, "max_cameras must be > 0, got 0");
        let s = services();
        let device = s.gpu.device();

        // Each camera takes 256 bytes (aligned requirement)
        let aligned_size: u64 = 256;
        let buffer_size = (max_cameras as u64) * aligned_size;

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // One uniform buffer per camera (dynamic offsets)
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    // size of one camera entry
                    min_binding_size: NonZeroU64::new(aligned_size),
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        });

        let per_element_size = NonZeroU64::new(aligned_size).unwrap();

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &camera_buffer,
                    offset: 0,
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

    /// Update a single camera’s GPU buffer (including tint)
    pub fn update(&self, i: usize, camera_uniform: &CameraUniform) {
        assert!(i < self.max_cameras);
        let s = services();
        let queue = s.gpu.queue();
        let aligned_size: u64 = 256;
        let offset = (i as u64) * aligned_size;

        // Write the entire CameraUniform, which now includes tint (e.g. vec4<f32>)
        queue.write_buffer(&self.camera_buffer, offset, bytemuck::cast_slice(&[*camera_uniform]));
    }

    /// Bind this camera’s data for a render pass
    pub fn bind(&self, pass: &mut wgpu::RenderPass, i: usize) {
        let aligned_size: u32 = 256;
        let offset = (i as u32) * aligned_size;
        pass.set_bind_group(1, &self.camera_bind_group, &[offset]);
    }
}
