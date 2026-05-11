use crate::{camera_rendering_components::CameraRenderingComponents, render_feature_3d::RenderFeature3D};
use curio_core::{
    LightSystem, Material, Matrix4x4, Mesh, TextureAsset, Vertex,
    built_in::record::{sys_record_lights::SysRecordLights, sys_record_rendering::SysRecordRendering, sys_record_sun::SysRecordSun},
    collections::ledger::Ledger,
    engine_services::services,
    system_adapters::adapter_system_gpu::SystemGPU,
};
use egui::ahash::{HashMap, HashMapExt};
use egui_wgpu::wgpu::{BindGroup, BindGroupLayout, BlendState, ColorTargetState, Device, FragmentState, RenderPass, RenderPipeline, ShaderModule, SurfaceConfiguration, util::DeviceExt};
use std::sync::Arc;

pub struct RenderFeatureDrawMesh {
    light_system: Vec<LightSystem>,
}

impl RenderFeatureDrawMesh {
    pub fn new() -> Box<RenderFeatureDrawMesh> {
        Box::new(RenderFeatureDrawMesh { light_system: Vec::new() })
    }

    fn draw_all_mesh(&mut self, ledger: &mut Ledger, config: &SurfaceConfiguration, device: &Device, render_pass: &mut RenderPass, camera: &CameraRenderingComponents, camera_index: usize, shadow_system_bind_group_layout: &BindGroupLayout, shadow_system_bind_group: &BindGroup) {
        let state_draws = ledger.read::<SysRecordRendering>();
        let mut draw_calls = state_draws.draw_calls.clone();
        let mut batching: HashMap<(Arc<Mesh>, Arc<Material>), Vec<Matrix4x4>> = HashMap::new();

        println!("{}", draw_calls.len());
        for draw_call in draw_calls.drain(..) {
            let mesh = draw_call.mesh;
            let material = draw_call.materials;
            let mut matrix = draw_call.matrix;

            if let Some(x) = batching.get_mut(&(mesh.clone(), material.clone())) {
                x.append(&mut matrix);
            } else {
                batching.insert((mesh, material), matrix);
            }
        }
        println!("{}", batching.len());

        // println!("Saved by batching {} => {}", was, batching.len());

        for ((mesh, material), matrix) in batching {
            println!("draw");
            self.draw_draw_call(mesh, material, matrix, config, device, render_pass, camera, camera_index, shadow_system_bind_group_layout, shadow_system_bind_group);
        }
    }

    fn draw_draw_call(
        &mut self,
        mesh: Arc<Mesh>,
        material: Arc<Material>,
        matrix: Vec<Matrix4x4>,
        config: &SurfaceConfiguration,
        device: &Device,
        render_pass: &mut RenderPass,
        camera: &CameraRenderingComponents,
        camera_index: usize,
        shadow_system_bind_group_layout: &BindGroupLayout,
        shadow_system_bind_group: &BindGroup,
    ) {
        // NOTE:
        // tint is assumed to be part of the camera uniform buffer (group 1 binding 0).
        // So we do NOT create a separate tint bind group here.
        // The camera component is expected to contain the tint field in its uniform buffer.

        // Create material bind groups
        let diffuse_bind_group = material.get_combined_binding_group();

        // Use layouts, not bind groups, for pipeline
        let rp = RenderFeatureDrawMesh::get_render_pipeline(
            &camera.camera_bind_group_layout,
            &diffuse_bind_group.1, // diffuse layout
            // &color_bind_group.1,                                // color layout
            &self.light_system[camera_index].bind_group_layout, // lights layout
            &shadow_system_bind_group_layout,
            config,
            device,
            material.shader(),
            false,
        );

        render_pass.set_pipeline(&rp);

        // Instance buffer
        let n_buffer = device.create_buffer_init(&egui_wgpu::wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&matrix),
            usage: egui_wgpu::wgpu::BufferUsages::VERTEX,
        });

        // Mesh vertex/index buffers
        let buffers = (mesh.get_vertex_buffer_for_device(), mesh.get_index_buffer_for_device());
        render_pass.set_vertex_buffer(0, buffers.0.slice(..));
        render_pass.set_vertex_buffer(1, n_buffer.slice(..));
        render_pass.set_index_buffer(buffers.1.slice(..), egui_wgpu::wgpu::IndexFormat::Uint32);

        // Set bind groups
        // group 0: diffuse texture/sampler (material)
        render_pass.set_bind_group(0, &diffuse_bind_group.0, &[]);
        // group 1: camera uniform bind group (now expected to include tint in the camera UB)
        render_pass.set_bind_group(1, &camera.camera_bind_group, &[(256 * camera_index).try_into().unwrap()]);
        // group 2: lights
        render_pass.set_bind_group(2, &self.light_system[camera_index].bind_group, &[]);
        // group 3: shadow system
        render_pass.set_bind_group(3, shadow_system_bind_group, &[]);
        // group 3: shadow system
        // render_pass.set_bind_group(4, &color_bind_group.0, &[]);

        // Draw
        render_pass.draw_indexed(0..(mesh.indicies.len() as u32), 0, 0..matrix.len() as u32);
        // }
    }

    fn get_render_pipeline(
        camera_bind: &BindGroupLayout,
        diffuse_bind_layout: &BindGroupLayout,
        // color_bind_layout: &BindGroupLayout,
        light_bind_layout: &BindGroupLayout,
        shadow_bind_layout: &BindGroupLayout,
        config: &SurfaceConfiguration,
        device: &Device,
        shader: Arc<ShaderModule>,
        wireframe: bool,
    ) -> RenderPipeline {
        let pipeline_layout = device.create_pipeline_layout(&egui_wgpu::wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                diffuse_bind_layout, // group 0
                camera_bind,         // group 1 (camera UB, which now includes tint)
                light_bind_layout,   // group 2
                shadow_bind_layout,  // group 3
                                     // color_bind_layout,
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
    fn render(&mut self, ledger: &mut Ledger, render_pass: &mut RenderPass, camera: &CameraRenderingComponents, camera_index: usize, shadow_system_bind_group_layout: &BindGroupLayout, shadow_system_bind_group: &BindGroup) {
        println!("rf draw mesh");
        while self.light_system.len() <= camera_index {
            self.light_system.push(LightSystem::new());
        }
        self.light_system[camera_index].update(&ledger.read::<SysRecordSun>().get_draw_call(), &ledger.read::<SysRecordLights>().all_lights);

        let s = services();
        let config = s.gpu.config();
        let device = s.gpu.device();

        self.draw_all_mesh(ledger, config, device, render_pass, camera, camera_index, shadow_system_bind_group_layout, shadow_system_bind_group);
    }

    fn clear(&mut self, ledger: &mut Ledger) {
        ledger.write::<SysRecordRendering>(|x| x.draw_calls.clear());
        ledger.write::<SysRecordLights>(|x| x.all_lights.clear());
    }
}
