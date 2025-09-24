use core::{collections::game_state::GameState, system_adapters::adapter_system_gpu::SystemGPU};
use std::sync::Arc;

use crate::render_feature_post_process::RenderFeaturePostProcess;

pub struct RenderFeaturePostProcessFog {
    pipeline: egui_wgpu::wgpu::RenderPipeline,
    bind_group: egui_wgpu::wgpu::BindGroup,
}

impl RenderFeaturePostProcessFog {
    pub fn new(device: Arc<egui_wgpu::wgpu::Device>, input_view: &egui_wgpu::wgpu::TextureView, sampler: &egui_wgpu::wgpu::Sampler, depth_view: &egui_wgpu::wgpu::TextureView) -> Box<Self> {
        let shader = device.create_shader_module(egui_wgpu::wgpu::ShaderModuleDescriptor {
            label: Some("Fog Shader"),
            source: egui_wgpu::wgpu::ShaderSource::Wgsl(include_str!("../postprocess_fog.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&egui_wgpu::wgpu::BindGroupLayoutDescriptor {
            label: Some("post BGL"),
            entries: &[
                egui_wgpu::wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: egui_wgpu::wgpu::ShaderStages::FRAGMENT,
                    ty: egui_wgpu::wgpu::BindingType::Sampler(egui_wgpu::wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                //
                egui_wgpu::wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: egui_wgpu::wgpu::ShaderStages::FRAGMENT,
                    ty: egui_wgpu::wgpu::BindingType::Texture {
                        sample_type: egui_wgpu::wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: egui_wgpu::wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // depth texture
                egui_wgpu::wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: egui_wgpu::wgpu::ShaderStages::FRAGMENT,
                    ty: egui_wgpu::wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: egui_wgpu::wgpu::TextureViewDimension::D2,
                        sample_type: egui_wgpu::wgpu::TextureSampleType::Depth,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&egui_wgpu::wgpu::BindGroupDescriptor {
            label: Some("post bind group"),
            layout: &bind_group_layout,
            entries: &[
                egui_wgpu::wgpu::BindGroupEntry {
                    binding: 0,
                    resource: egui_wgpu::wgpu::BindingResource::Sampler(sampler),
                },
                egui_wgpu::wgpu::BindGroupEntry {
                    binding: 1,
                    resource: egui_wgpu::wgpu::BindingResource::TextureView(input_view),
                },
                egui_wgpu::wgpu::BindGroupEntry {
                    binding: 2,
                    resource: egui_wgpu::wgpu::BindingResource::TextureView(&depth_view),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&egui_wgpu::wgpu::PipelineLayoutDescriptor {
            label: Some("post pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&egui_wgpu::wgpu::RenderPipelineDescriptor {
            label: Some("postprocess pipeline"),
            layout: Some(&pipeline_layout),
            vertex: egui_wgpu::wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_fullscreen"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(egui_wgpu::wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_fullscreen"),
                targets: &[Some(egui_wgpu::wgpu::ColorTargetState {
                    format: SystemGPU::get_config().format,
                    blend: Some(egui_wgpu::wgpu::BlendState::REPLACE),
                    write_mask: egui_wgpu::wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: egui_wgpu::wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: egui_wgpu::wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Box::new(Self { pipeline, bind_group })
    }
}

impl RenderFeaturePostProcess for RenderFeaturePostProcessFog {
    fn render(&mut self, _game_state: &mut GameState, render_pass: &mut egui_wgpu::wgpu::RenderPass<'_>, _input_view: &egui_wgpu::wgpu::TextureView) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..3, 0..1); // full-screen triangle
    }

    fn clear(&mut self, _game_state: &mut GameState) {}
}
