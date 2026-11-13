use curio_core::Services;
use egui_wgpu::wgpu::{self, BindGroup};
use egui_wgpu::wgpu::util::DeviceExt;
use std::borrow::Cow;
use std::num::NonZeroU64;
use std::sync::Arc;

use bytemuck::{Pod, Zeroable, cast_slice};

use crate::PostProcessResources;
use crate::render_feature_post_process::{PostProcessSource, RenderFeaturePostProcess};

// NOTE: update include_str! path to your shader file
// const FOG_WGSL_PATH: &str = "../postprocess_fog.wgsl";

/// CPU-side mirror of the WGSL KuwaharaParams struct
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct KuwaharaUniform {
    radius: f32, // kernel radius in pixels
    strength: f32, // mix between the original color and the painterly color
}

pub struct RenderFeaturePostProcessKuwahara {
    pipeline: wgpu::RenderPipeline,
    bind_group_input: wgpu::BindGroup, // samples the original offscreen_view
    bind_group_a: wgpu::BindGroup, // samples view_a
    bind_group_b: wgpu::BindGroup, // samples view_b
    params: wgpu::Buffer, // KuwaharaUniform, rewritten per frame
    params_uniform: KuwaharaUniform,
}

impl RenderFeaturePostProcessKuwahara {
    pub fn new(
        device: Arc<wgpu::Device>,
        format: wgpu::TextureFormat,
        post_resources: &PostProcessResources,
        depth_view: &wgpu::TextureView,
        // add offscreen_view here so we can create the initial bind group
        offscreen_view: &wgpu::TextureView,
    ) -> Box<Self> {
        // default parameters - the naive filter's radius of 4, fully painterly
        let params_uniform = KuwaharaUniform { radius: 4.0, strength: 1.0 };

        // sampler
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("kuwahara sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("kuwahara shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../postprocess_kuwahara.wgsl"))),
        });

        // params uniform buffer
        let params = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("kuwahara params"),
            contents: cast_slice(&[params_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // bind group layout (sampler, color texture, depth texture, params)
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("kuwahara bind group layout"),
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
                // kuwahara parameters
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(8),
                    },
                    count: None,
                },
            ],
        });

        // pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("kuwahara pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("kuwahara pipeline"),
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
            label: Some("kuwahara bind group INPUT"),
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
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &params,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        // bind_group_a: sample from post_resources.view_a
        let bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("kuwahara bind group A"),
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
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &params,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        // bind_group_b: sample from post_resources.view_b
        let bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("kuwahara bind group B"),
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
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &params,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        // store raw pointer ids for quick comparison later
        // let bind_group_input_tex_id = (offscreen_view as *const _) as usize;
        // let bind_group_a_tex_id = (&post_resources.view_a as *const _) as usize;
        // let bind_group_b_tex_id = (&post_resources.view_b as *const _) as usize;

        Box::new(Self {
            pipeline,
            bind_group_input,
            bind_group_a,
            bind_group_b,
            params,
            params_uniform,
            // bind_group_input_tex_id,
            // bind_group_a_tex_id,
            // bind_group_b_tex_id,
        })
    }

    // rewrite the params uniform, only when something actually changed
    pub fn update(&mut self, radius: f32, strength: f32) {
        let radius = radius.max(1.0);
        let strength = strength.clamp(0.0, 1.0);

        if self.params_uniform.radius == radius && self.params_uniform.strength == strength {
            return;
        }

        self.params_uniform = KuwaharaUniform { radius, strength };

        let s = Services::get();
        let queue = s.gpu.queue();
        queue.write_buffer(&self.params, 0, cast_slice(&[self.params_uniform]));
    }
}

impl RenderFeaturePostProcess for RenderFeaturePostProcessKuwahara {
    fn render(&mut self, encoder: &mut wgpu::CommandEncoder, _input_view: &wgpu::TextureView, output_view: &wgpu::TextureView, postprocess_source: PostProcessSource) {
        let bind_group: &BindGroup;
        match postprocess_source {
            PostProcessSource::Offscreen => bind_group = &self.bind_group_input,
            PostProcessSource::ViewA => bind_group = &self.bind_group_a,
            PostProcessSource::ViewB => bind_group = &self.bind_group_b,
        }

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("kuwahara pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(egui_wgpu::wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}
