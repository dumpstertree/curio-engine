use ::core::collections::matrix4x4::Matrix4x4;
use ::core::system_adapters::adapter_system_gpu::SystemGPU;
use core::collections::{draw_call::DrawCall, mesh::Vertex, quaternion::Quaternion, vector3::Vector3, vector4::Vector4};
use std::num::NonZeroU64;

use bytemuck::bytes_of;
use egui_wgpu::wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBindingType, BufferUsages, CommandEncoder, CompareFunction, DepthBiasState, Extent3d, FilterMode, RenderPipeline, Sampler,
    SamplerBindingType, SamplerDescriptor, ShaderStages, Texture, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
    util::{self, DeviceExt},
};

pub const SHADOW_SIZE: u32 = 2048;

pub struct ShadowSystem {
    pub shadow_pipeline: RenderPipeline,
    pub bind_group: BindGroup,
    pub bind_group_layout: BindGroupLayout,
    pub buffer: Buffer,
    pub depth_texture: Texture,
    pub depth_view: TextureView,
    pub sampler: Sampler,
    pub light_matrix_bind_group: BindGroup,
    pub light_matrix_bind_group_layout: BindGroupLayout,
}

// ... earlier imports and constants unchanged ...

impl ShadowSystem {
    pub fn new(initial_light_view_proj: Matrix4x4) -> Self {
        let device = SystemGPU::get_device();

        // 1) buffer (light view-proj)
        let buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("shadow camera buffer"),
            contents: bytemuck::bytes_of(&initial_light_view_proj),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        // 2) depth texture (shadow map)
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
        let depth_view = depth_texture.create_view(&TextureViewDescriptor { aspect: TextureAspect::DepthOnly, ..Default::default() });

        // 3) comparison sampler (for sampling shadow map in main pass)
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

        // -------------------------
        // 4) Layout: matrix only (for shadow-pass pipeline)
        // -------------------------
        let matrix_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("shadow.matrix.layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX, // vertex shader only for depth pass
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(std::mem::size_of::<Matrix4x4>() as u64),
                },
                count: None,
            }],
        });

        // matrix bind group (for shadow pass)
        let matrix_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("shadow.matrix.bind_group"),
            layout: &matrix_layout,
            entries: &[BindGroupEntry { binding: 0, resource: buffer.as_entire_binding() }],
        });

        // -------------------------
        // 5) Layout: sampling (matrix + shadow_map + sampler) (for main pass)
        // -------------------------
        let sampling_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("shadow.sampling.layout"),
            entries: &[
                // 0 = matrix (also needed for sampling shader)
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
                // 1 = depth texture (shadow map)
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
                // 2 = comparison sampler
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Comparison),
                    count: None,
                },
            ],
        });

        // sampling bind group (for main pass)
        let sampling_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("shadow.sampling.bind_group"),
            layout: &sampling_layout,
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

        // -------------------------
        // 6) Shadow pipeline uses matrix_layout only (depth-only)
        // -------------------------
        let shadow_shader = device.create_shader_module(egui_wgpu::wgpu::ShaderModuleDescriptor {
            label: Some("shadow shader"),
            source: egui_wgpu::wgpu::ShaderSource::Wgsl(include_str!("shadow_pass.wgsl").into()),
        });

        let shadow_pipeline_layout = device.create_pipeline_layout(&egui_wgpu::wgpu::PipelineLayoutDescriptor {
            label: Some("shadow pipeline layout"),
            bind_group_layouts: &[&matrix_layout], // IMPORTANT: only matrix layout here
            push_constant_ranges: &[],
        });

        let shadow_pipeline = device.create_render_pipeline(&egui_wgpu::wgpu::RenderPipelineDescriptor {
            label: Some("shadow pipeline"),
            layout: Some(&shadow_pipeline_layout),
            vertex: egui_wgpu::wgpu::VertexState {
                module: &shadow_shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), Matrix4x4::desc()],
                compilation_options: Default::default(),
            },
            fragment: None, // depth-only pass
            primitive: egui_wgpu::wgpu::PrimitiveState {
                cull_mode: None, // ensure nothing gets culled
                ..Default::default()
            },
            depth_stencil: Some(egui_wgpu::wgpu::DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: egui_wgpu::wgpu::CompareFunction::LessEqual,
                stencil: egui_wgpu::wgpu::StencilState::default(),
                bias: DepthBiasState { constant: 2, slope_scale: 2.0, clamp: 0.0 },
            }),
            multisample: egui_wgpu::wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            shadow_pipeline,
            // we keep sampling_layout and matrix_layout around so caller can include sampling_layout in main pipeline creation
            bind_group: sampling_bind_group,    // use this in main pass to sample
            bind_group_layout: sampling_layout, // layout to be inserted into main pipeline
            buffer,
            depth_texture,
            depth_view,
            sampler,
            // store matrix bind group separately so shadow pass can bind it:
            light_matrix_bind_group: matrix_bind_group,
            light_matrix_bind_group_layout: matrix_layout,
        }
    }

    // ... update() method unchanged (writes buffer) ...

    /// Recompute light matrix each frame
    pub fn update(&self, t: f32) {
        let queue = SystemGPU::get_queue();

        let pos = f32::sin(t) * 5.0;
        println!("pos {}", pos);
        let light_pos = Vector3::new(pos, 10.0, -10.0); // example
        let target = Vector3::zero(); // look at world origin
        let up = Vector3::up();

        let light_view = Matrix4x4::look_at(light_pos, target, up);
        let light_proj = Matrix4x4::orthographic_lh_zo(-20.0, 20.0, -20.0, 20.0, 0.1, 200.0);
        let light_view_proj = Matrix4x4::multiply(&light_view, &light_proj);

        let world_origin = Vector4::new(0.0, 0.0, 0.0, 1.0);
        let clip = light_view_proj.multiply_vec4(world_origin);

        // compute NDC coordinates (watch out for w == 0)
        if clip.w.abs() > 1e-9 {
            let ndc_x = clip.x / clip.w;
            let ndc_y = clip.y / clip.w;
            let ndc_z = clip.z / clip.w;
            println!("clip = {:?}, ndc = ({:.6}, {:.6}, {:.6})", clip, ndc_x, ndc_y, ndc_z);
        } else {
            println!("clip.w is ~0 -> invalid projection: clip = {:?}", clip);
        }
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&light_view_proj));
    }

    /// Render depth from the light’s perspective
    pub fn render(&self, encoder: &mut CommandEncoder, draw_calls: &[DrawCall]) {
        println!("Shadow render: draw_calls.len={} total_instances={}", draw_calls.len(), draw_calls.iter().map(|d| d.matrix.len()).sum::<usize>());

        let device = SystemGPU::get_device();
        let mut shadow_pass = encoder.begin_render_pass(&egui_wgpu::wgpu::RenderPassDescriptor {
            label: Some("shadow pass"),
            color_attachments: &[],
            depth_stencil_attachment: Some(egui_wgpu::wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_view,
                depth_ops: Some(egui_wgpu::wgpu::Operations {
                    load: egui_wgpu::wgpu::LoadOp::Clear(1.0),
                    store: egui_wgpu::wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        shadow_pass.set_pipeline(&self.shadow_pipeline);

        // bind matrix-only group (no texture/sampler)
        shadow_pass.set_bind_group(0, &self.light_matrix_bind_group, &[]);

        for draw_call in draw_calls {
            // if !draw_call.matrix.is_empty() {
            //     let m = &draw_call.matrix[0];
            //     println!("Instance matrix[0] first column: {:?}", m.model[0]);
            // }
            for mesh in &draw_call.mesh {
                let n_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: bytemuck::cast_slice(&draw_call.matrix),
                    usage: BufferUsages::VERTEX,
                });

                shadow_pass.set_vertex_buffer(0, mesh.get_vertex_buffer_for_device().slice(..));
                shadow_pass.set_vertex_buffer(1, n_buffer.slice(..));
                shadow_pass.set_index_buffer(mesh.get_index_buffer_for_device().slice(..), egui_wgpu::wgpu::IndexFormat::Uint32);

                // draw
                shadow_pass.draw_indexed(0..(mesh.indicies.len() as u32), 0, 0..draw_call.matrix.len() as u32);
            }
        }
    }
}
