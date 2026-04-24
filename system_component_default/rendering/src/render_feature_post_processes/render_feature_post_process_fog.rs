use egui_wgpu::wgpu::{self, BindGroup};
use std::borrow::Cow;
use std::sync::Arc;

use crate::PostProcessResources;
use crate::render_feature_post_process::{PostProcessSource, RenderFeaturePostProcess};

// NOTE: update include_str! path to your shader file
const FOG_WGSL_PATH: &str = "../postprocess_fog.wgsl";

pub struct RenderFeaturePostProcessFog {
    pipeline: wgpu::RenderPipeline,
    bind_group_input: wgpu::BindGroup, // samples the original offscreen_view
    bind_group_a: wgpu::BindGroup,     // samples view_a
    bind_group_b: wgpu::BindGroup,     // samples view_b
    bind_group_input_tex_id: usize,
    bind_group_a_tex_id: usize,
    bind_group_b_tex_id: usize,
}

impl RenderFeaturePostProcessFog {
    pub fn new(
        device: Arc<wgpu::Device>,
        format: wgpu::TextureFormat,
        post_resources: &PostProcessResources,
        depth_view: &wgpu::TextureView,
        // add offscreen_view here so we can create the initial bind group
        offscreen_view: &wgpu::TextureView,
    ) -> Box<Self> {
        // sampler
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("fog sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fog shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../postprocess_fog.wgsl"))),
        });

        // bind group layout (sampler, color texture, depth texture)
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fog bind group layout"),
            entries: &[
                // sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // color texture (float)
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // depth texture (depth)
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        // pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fog pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fog pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_fullscreen"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_fullscreen"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // bind_group_input: sample from offscreen_view (initial scene)
        let bind_group_input = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fog bind group INPUT"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(offscreen_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(depth_view),
                },
            ],
        });

        // bind_group_a: sample from post_resources.view_a
        let bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fog bind group A"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&post_resources.view_a),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(depth_view),
                },
            ],
        });

        // bind_group_b: sample from post_resources.view_b
        let bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fog bind group B"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&post_resources.view_b),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(depth_view),
                },
            ],
        });

        // store raw pointer ids for quick comparison later
        let bind_group_input_tex_id = (offscreen_view as *const _) as usize;
        let bind_group_a_tex_id = (&post_resources.view_a as *const _) as usize;
        let bind_group_b_tex_id = (&post_resources.view_b as *const _) as usize;

        Box::new(Self {
            pipeline,
            bind_group_input,
            bind_group_a,
            bind_group_b,
            bind_group_input_tex_id,
            bind_group_a_tex_id,
            bind_group_b_tex_id,
        })
    }
}

impl RenderFeaturePostProcess for RenderFeaturePostProcessFog {
    fn render(&mut self, encoder: &mut wgpu::CommandEncoder, _input_view: &wgpu::TextureView, output_view: &wgpu::TextureView, postprocess_source: PostProcessSource) {
        let bind_group: &BindGroup;
        match postprocess_source {
            PostProcessSource::Offscreen => bind_group = &self.bind_group_input,
            PostProcessSource::ViewA => bind_group = &self.bind_group_a,
            PostProcessSource::ViewB => bind_group = &self.bind_group_b,
        }

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("fog pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output_view,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, bind_group, &[]);
        pass.draw(0..3, 0..1);
    }

    fn clear(&mut self, _ledger: &mut crate::Ledger) {}
}
