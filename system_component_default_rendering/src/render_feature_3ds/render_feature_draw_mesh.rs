use crate::{camera_rendering_components::CameraRenderingComponents, render_feature_3d::RenderFeature3D};
use built_in_state::{state_draw::DrawCallsState, state_lights::StateLights};
use core::{
    collections::{
        draw_call::DrawCall,
        game_state::{self, GameState},
        light_uniform::LightSystem,
        matrix4x4::Matrix4x4,
        mesh::Vertex,
    },
    io::texture_asset::TextureAsset,
    system_adapters::adapter_system_gpu::SystemGPU,
};
use egui_wgpu::wgpu::{BindGroupLayout, BlendState, ColorTargetState, Device, FragmentState, RenderPass, RenderPipeline, ShaderModule, SurfaceConfiguration, util::DeviceExt};
use std::sync::Arc;

pub struct RenderFeatureDrawMesh {
    light_system: LightSystem,
}

impl RenderFeatureDrawMesh {
    pub fn new() -> Box<RenderFeatureDrawMesh> {
        Box::new(RenderFeatureDrawMesh { light_system: LightSystem::new(16) })
    }

    fn draw_all_mesh(&mut self, game_state: &mut GameState, config: &Arc<SurfaceConfiguration>, device: &Arc<Device>, render_pass: &mut RenderPass, camera: &CameraRenderingComponents, camera_index: usize) {
        let state_draws = game_state.get_value2::<DrawCallsState>();
        let draw_calls = &state_draws.draw_calls;

        for draw_call in draw_calls {
            self.draw_draw_call(draw_call, config, device, render_pass, camera, camera_index);
        }
    }

    fn draw_draw_call(&mut self, draw_call: &DrawCall, config: &SurfaceConfiguration, device: &Device, render_pass: &mut RenderPass, camera: &CameraRenderingComponents, camera_index: usize) {
        for i in 0..draw_call.mesh.len() {
            let mesh = &draw_call.mesh[i];
            let material = &draw_call.materials[i];

            // Create material bind groups
            let diffuse_bind_group = material.get_texture_binding_group(device);
            let color_bind_group = material.get_color_binding_group(device);

            // Use layouts, not bind groups, for pipeline
            let rp = RenderFeatureDrawMesh::get_render_pipeline(
                &camera.camera_bind_group_layout,
                &diffuse_bind_group.1,                // diffuse layout
                &color_bind_group.1,                  // color layout
                &self.light_system.bind_group_layout, // lights layout
                config,
                device,
                material.shader.clone(),
                false,
            );

            render_pass.set_pipeline(&rp);

            // Instance buffer
            let n_buffer = device.create_buffer_init(&egui_wgpu::wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&draw_call.matrix),
                usage: egui_wgpu::wgpu::BufferUsages::VERTEX,
            });

            // Mesh vertex/index buffers
            let buffers = (mesh.get_vertex_buffer_for_device(), mesh.get_index_buffer_for_device());
            render_pass.set_vertex_buffer(0, buffers.0.slice(..));
            render_pass.set_vertex_buffer(1, n_buffer.slice(..));
            render_pass.set_index_buffer(buffers.1.slice(..), egui_wgpu::wgpu::IndexFormat::Uint32);

            // Set bind groups
            render_pass.set_bind_group(0, &diffuse_bind_group.0, &[]);
            render_pass.set_bind_group(1, &camera.camera_bind_group, &[(256 * camera_index).try_into().unwrap()]);
            // render_pass.set_bind_group(2, &color_bind_group.0, &[]);
            render_pass.set_bind_group(2, &self.light_system.bind_group, &[]);

            // Draw
            render_pass.draw_indexed(0..(mesh.indicies.len() as u32), 0, 0..draw_call.matrix.len() as u32);
        }
    }

    fn get_render_pipeline(camera_bind: &BindGroupLayout, diffuse_bind_layout: &BindGroupLayout, color_bind_layout: &BindGroupLayout, light_bind_layout: &BindGroupLayout, config: &SurfaceConfiguration, device: &Device, shader: ShaderModule, wireframe: bool) -> RenderPipeline {
        let pipeline_layout = device.create_pipeline_layout(&egui_wgpu::wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                diffuse_bind_layout, // group 0
                camera_bind,         // group 1
                // color_bind_layout,   // group 2
                light_bind_layout, // group 3
            ],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&egui_wgpu::wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: egui_wgpu::wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), Matrix4x4::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: egui_wgpu::wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: egui_wgpu::wgpu::PrimitiveState {
                topology: egui_wgpu::wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: egui_wgpu::wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: if wireframe { egui_wgpu::wgpu::PolygonMode::Line } else { egui_wgpu::wgpu::PolygonMode::Fill },
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(egui_wgpu::wgpu::DepthStencilState {
                format: TextureAsset::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: egui_wgpu::wgpu::CompareFunction::Less,
                stencil: egui_wgpu::wgpu::StencilState::default(),
                bias: egui_wgpu::wgpu::DepthBiasState::default(),
            }),
            multisample: egui_wgpu::wgpu::MultisampleState { count: 1, mask: !0, alpha_to_coverage_enabled: false },
            multiview: None,
            cache: None,
        })
    }
}

impl RenderFeature3D for RenderFeatureDrawMesh {
    fn render(&mut self, game_state: &mut GameState, render_pass: &mut RenderPass, camera: &CameraRenderingComponents, camera_index: usize) {
        self.light_system
            .update(&game_state.get_value2::<StateLights>().all_lights);

        let config = SystemGPU::get_config();
        let device = SystemGPU::get_device();

        self.draw_all_mesh(game_state, &config, &device, render_pass, camera, camera_index);
    }

    fn clear(&mut self, game_state: &mut GameState) {
        game_state.edit::<DrawCallsState>(|x| x.draw_calls.clear());
        game_state.edit::<StateLights>(|x| x.all_lights.clear());
    }
}
