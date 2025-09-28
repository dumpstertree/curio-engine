use ::core::collections::matrix4x4::Matrix4x4;
use ::core::system_adapters::adapter_system_gpu::SystemGPU;
use std::num::NonZeroU64;

use egui_wgpu::wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBindingType, BufferUsages, CompareFunction, Extent3d, FilterMode, Sampler, SamplerBindingType, SamplerDescriptor,
    ShaderStages, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
    util::{self, DeviceExt},
};
pub const SHADOW_SIZE: u32 = 2048;

pub struct ShadowSystem {
    pub bind_group: BindGroup,
    pub bind_group_layout: BindGroupLayout,
    pub buffer: Buffer,
    pub depth_texture: Texture,
    pub depth_view: TextureView,
    pub sampler: Sampler,
}

impl ShadowSystem {
    pub fn new(light_view_proj: Matrix4x4) -> Self {
        let device = SystemGPU::get_device();

        // ------------------ Uniform buffer for light VP ------------------
        // let shadow_camera = ShadowCamera { view_proj: light_view_proj };
        let buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("shadow camera buffer"),
            contents: bytemuck::bytes_of(&light_view_proj),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        // ------------------ Depth texture ------------------
        let depth_texture = device.create_texture(&TextureDescriptor {
            label: Some("shadow depth texture"),
            size: Extent3d {
                width: SHADOW_SIZE,
                height: SHADOW_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&TextureViewDescriptor::default());

        // ------------------ Comparison sampler ------------------
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("shadow comparison sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            compare: Some(CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        // ------------------ Bind group layout ------------------
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("shadow bind group layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(std::mem::size_of::<Matrix4x4>() as u64),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Depth,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Comparison),
                    count: None,
                },
            ],
        });

        // ------------------ Bind group ------------------
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("shadow bind group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry { binding: 0, resource: buffer.as_entire_binding() },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&depth_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            bind_group,
            bind_group_layout,
            buffer,
            depth_texture,
            depth_view,
            sampler,
        }
    }

    /// Update the shadow camera buffer each frame
    pub fn update(&self, light_view_proj: &Matrix4x4) {
        let queue = SystemGPU::get_queue();
        // let shadow_camera = ShadowCamera { view_proj: light_view_proj.model };

        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(light_view_proj));
    }
}
