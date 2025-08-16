use crate::render_feature_3d::RenderFeature3D;
use core::{
    collections::{draw_call::DrawCall, game_state::GameState, matrix4x4::Matrix4x4, mesh::Vertex},
    io::texture_asset::TextureAsset,
    system::system_game_states::state_draw::DrawCallsState,
    system_adapters::adapter_system_gpu::SystemGPU,
};
use egui_wgpu::wgpu::{
    BindGroup, BindGroupLayout, BlendState, ColorTargetState, Device, FragmentState, RenderPass, RenderPipeline, ShaderModule, SurfaceConfiguration,
    util::DeviceExt,
};
use std::sync::Arc;

pub struct RenderFeatureDrawMesh {}
impl RenderFeatureDrawMesh {
    pub fn new() -> Box<RenderFeatureDrawMesh> {
        Box::new(RenderFeatureDrawMesh {})
    }
    fn draw_all_mesh(
        &mut self,
        game_state: &mut GameState,
        config: &Arc<SurfaceConfiguration>,
        device: &Arc<Device>,
        render_pass: &mut RenderPass,
        camera_bind: &BindGroup,
        camera_bind_layout: &BindGroupLayout,
    ) {
        let state_draws = game_state.get_value2::<DrawCallsState>();
        let draw_calls = &state_draws.draw_calls;
        for draw_call in draw_calls {
            self.draw_draw_call(draw_call, config, device, render_pass, camera_bind, camera_bind_layout);
        }

        // clear state
        // game_state.edit::<DrawCallsState>(|x| {
        //     x.draw_calls.clear();
        // });
    }
    fn draw_draw_call(
        &mut self,
        draw_call: &DrawCall,
        config: &SurfaceConfiguration,
        device: &Device,
        render_pass: &mut RenderPass,
        camera_bind: &BindGroup,
        camera_bind_layout: &BindGroupLayout,
    ) {
        // iterate over each mesh in the draw call
        for i in 0..draw_call.mesh.len() {
            //
            let mesh = &draw_call.mesh[i];
            let material = &draw_call.materials[i];
            // create material bind group
            // let diffuse_bind_group = egui_wgpu::wgpuGraphicsComponent::get_diffuse_binding(&state, &material.textures[..]);
            let diffuse_bind_group = material.get_texture_binding_group(device);
            let color_bind_group = material.get_color_binding_group(device);
            // set render pipeline
            let rp = RenderFeatureDrawMesh::get_render_pipeline(
                camera_bind_layout,
                config,
                device,
                material.shader.clone(),
                &diffuse_bind_group.1,
                &color_bind_group.1,
                false,
            );
            render_pass.set_pipeline(&rp);

            // create the instance buffer
            let n_buffer = device.create_buffer_init(&egui_wgpu::wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&draw_call.matrix),
                usage: egui_wgpu::wgpu::BufferUsages::VERTEX,
            });
            // fetch the cached buffers for the mesh
            // let buffers: (&Buffer, &Buffer) = self.buffer_cache.get_vertex_buffer(device, mesh);
            let buffers = (mesh.get_vertex_buffer_for_device(device), mesh.get_index_buffer_for_device(device));

            // set buffers
            render_pass.set_vertex_buffer(0, buffers.0.slice(..));
            render_pass.set_vertex_buffer(1, n_buffer.slice(..));
            render_pass.set_index_buffer(buffers.1.slice(..), egui_wgpu::wgpu::IndexFormat::Uint32);

            // set bind groups
            render_pass.set_bind_group(0, &diffuse_bind_group.0, &[]);
            render_pass.set_bind_group(1, camera_bind, &[]);
            render_pass.set_bind_group(2, &color_bind_group.0, &[]);

            // draw
            render_pass.draw_indexed(0..(mesh.indicies.len() as u32), 0, 0..draw_call.matrix.len() as u32);
        }
    }
    fn get_render_pipeline(
        camera_bind: &BindGroupLayout,
        config: &SurfaceConfiguration,
        device: &Device,
        shader: ShaderModule,
        texture_bind_group_layout: &BindGroupLayout,
        color_bind_group_layout: &BindGroupLayout,
        wireframe: bool,
    ) -> RenderPipeline {
        let render_pipeline_layout = device.create_pipeline_layout(&egui_wgpu::wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &camera_bind, &color_bind_group_layout],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&egui_wgpu::wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: egui_wgpu::wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                // buffers: &[super::model::ModelVertex::desc(), InstanceRaw::desc()],
                buffers: &[Vertex::desc(), Matrix4x4::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState {
                        color: egui_wgpu::wgpu::BlendComponent::REPLACE,
                        alpha: egui_wgpu::wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: egui_wgpu::wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: egui_wgpu::wgpu::PrimitiveState {
                topology: egui_wgpu::wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: egui_wgpu::wgpu::FrontFace::Ccw,
                cull_mode: Some(egui_wgpu::wgpu::Face::Back),
                polygon_mode: if wireframe {
                    egui_wgpu::wgpu::PolygonMode::Line
                } else {
                    egui_wgpu::wgpu::PolygonMode::Fill
                },
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
            // depth_stencil: None,
            multisample: egui_wgpu::wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
            // Useful for optimizing shader compilation on Android
            cache: None,
        })
    }
}
impl RenderFeature3D for RenderFeatureDrawMesh {
    fn render(
        &mut self,
        game_state: &mut GameState,
        render_pass: &mut RenderPass,
        camera_bind_group: &BindGroup,
        camera_bind_group_layout: &BindGroupLayout,
    ) {
        // get system values
        let config = SystemGPU::get_config();
        let device = SystemGPU::get_device();
        // draw all
        self.draw_all_mesh(game_state, &config, &device, render_pass, &camera_bind_group, &camera_bind_group_layout);
    }
    fn clear(&mut self, game_state: &mut GameState) {
        game_state.edit::<DrawCallsState>(|x| {
            x.draw_calls.clear();
        });
    }
}
