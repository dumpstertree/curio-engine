use crate::{camera_rendering_components::CameraRenderingComponents, render_feature_3d::RenderFeature3D};
use curio_core::{Ledger, Matrix4x4, TextureAsset, services};
use egui_wgpu::wgpu::{BindGroup, BindGroupLayout, BlendState, Buffer, BufferDescriptor, BufferUsages, ColorTargetState, Device, Face, FragmentState, RenderPass, RenderPipeline, ShaderModule, SurfaceConfiguration, util::DeviceExt};
use ext_rendering::{Material, Mesh, SysRecordRendering, data::mesh::Vertex};
use lighting::{LightSystem, SysRecordLights, SysRecordSun};
use std::{collections::HashMap, sync::Arc, time::Instant};

pub struct RenderFeatureDrawMesh {
    light_system: Vec<LightSystem>,
    pipeline_cache: HashMap<PipelineCacheKey, RenderPipeline>,
    instance_buffer: Option<Buffer>,
    instance_buffer_capacity: usize,
}

#[derive(Hash, PartialEq, Eq)]
struct PipelineCacheKey {
    shader_id: usize, // some stable ID for the Arc<ShaderModule>
    wireframe: bool,
}

impl RenderFeatureDrawMesh {
    pub fn new() -> Box<RenderFeatureDrawMesh> {
        let b = Box::new(RenderFeatureDrawMesh {
            light_system: Vec::new(),
            pipeline_cache: HashMap::default(),
            instance_buffer: None,
            instance_buffer_capacity: 0,
        });

        b
    }

    pub fn prewarm_pipelines(&mut self, shaders: &[Arc<ShaderModule>], camera_bind: &BindGroupLayout, diffuse_bind_layout: &BindGroupLayout, light_bind_layout: &BindGroupLayout, shadow_bind_layout: &BindGroupLayout, config: &SurfaceConfiguration, device: &Device) {
        for shader in shaders {
            for wireframe in [false, true] {
                let key = PipelineCacheKey { shader_id: Arc::as_ptr(shader) as usize, wireframe };
                self.pipeline_cache
                    .entry(key)
                    .or_insert_with(|| RenderFeatureDrawMesh::get_render_pipeline(camera_bind, diffuse_bind_layout, light_bind_layout, shadow_bind_layout, config, device, shader.clone(), wireframe));
            }
        }
    }
    fn draw_all_mesh(&mut self, ledger: &mut Ledger, config: &SurfaceConfiguration, device: &Device, render_pass: &mut RenderPass, camera: &CameraRenderingComponents, camera_index: usize, shadow_system_bind_group_layout: &BindGroupLayout, shadow_system_bind_group: &BindGroup) {
        // prewarm

        let state_draws = ledger.read::<SysRecordRendering>();
        let mut draw_calls = state_draws.draw_calls.clone();
        let mut batching: HashMap<(Arc<Mesh>, Arc<Material>), Vec<Matrix4x4>> = HashMap::new();

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

        let total_instances: usize = batching.values().map(|v| v.len()).sum();

        if self.instance_buffer.is_none() || self.instance_buffer_capacity < total_instances {
            self.instance_buffer = Some(device.create_buffer(&BufferDescriptor {
                label: Some("Instance Buffer"),
                size: (total_instances * std::mem::size_of::<Matrix4x4>()) as u64,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
            self.instance_buffer_capacity = total_instances;
        }

        let queue = services().gpu.queue();
        let mut offset = 0;
        let mut batches_with_offsets: Vec<(Arc<Mesh>, Arc<Material>, Vec<Matrix4x4>, usize)> = Vec::new();

        for ((mesh, material), matrix) in &batching {
            queue.write_buffer(self.instance_buffer.as_ref().unwrap(), (offset * std::mem::size_of::<Matrix4x4>()) as u64, bytemuck::cast_slice(matrix));
            batches_with_offsets.push((mesh.clone(), material.clone(), matrix.clone(), offset));
            offset += matrix.len();
        }

        // Sort by shader_id to minimize set_pipeline calls
        batches_with_offsets.sort_by_key(|(_, material, _, _)| Arc::as_ptr(&material.shader()) as usize);

        let mut last_shader_id: Option<usize> = None;

        for (mesh, material, matrix, instance_offset) in batches_with_offsets {
            self.draw_draw_call(mesh, material, matrix, instance_offset, &mut last_shader_id, config, device, render_pass, camera, camera_index, shadow_system_bind_group_layout, shadow_system_bind_group);
        }
    }

    fn draw_draw_call(
        &mut self,
        mesh: Arc<Mesh>,
        material: Arc<Material>,
        matrix: Vec<Matrix4x4>,
        instance_offset: usize,
        last_shader_id: &mut Option<usize>,
        config: &SurfaceConfiguration,
        device: &Device,
        render_pass: &mut RenderPass,
        camera: &CameraRenderingComponents,
        camera_index: usize,
        shadow_system_bind_group_layout: &BindGroupLayout,
        shadow_system_bind_group: &BindGroup,
    ) {
        let diffuse_bind_group = material.get_combined_binding_group();
        let shader_id = Arc::as_ptr(&material.shader()) as usize;

        if *last_shader_id != Some(shader_id) {
            let rp = self
                .pipeline_cache
                .entry(PipelineCacheKey { shader_id, wireframe: false })
                .or_insert_with(|| {
                    RenderFeatureDrawMesh::get_render_pipeline(
                        &camera.camera_bind_group_layout,
                        &diffuse_bind_group.1,
                        &self.light_system[camera_index].bind_group_layout,
                        &shadow_system_bind_group_layout,
                        config,
                        device,
                        material.shader(),
                        false,
                    )
                });
            render_pass.set_pipeline(rp);
            *last_shader_id = Some(shader_id);
        }

        let n_buffer = self.instance_buffer.as_ref().unwrap();
        let byte_offset = (instance_offset * std::mem::size_of::<Matrix4x4>()) as u64;

        let buffers = (mesh.get_vertex_buffer_for_device(), mesh.get_index_buffer_for_device());
        render_pass.set_vertex_buffer(0, buffers.0.slice(..));
        render_pass.set_vertex_buffer(1, n_buffer.slice(byte_offset..));
        render_pass.set_index_buffer(buffers.1.slice(..), egui_wgpu::wgpu::IndexFormat::Uint32);

        render_pass.set_bind_group(0, &diffuse_bind_group.0, &[]);
        render_pass.set_bind_group(1, &camera.camera_bind_group, &[(256 * camera_index).try_into().unwrap()]);
        render_pass.set_bind_group(2, &self.light_system[camera_index].bind_group, &[]);
        render_pass.set_bind_group(3, shadow_system_bind_group, &[]);

        render_pass.draw_indexed(0..(mesh.indicies.len() as u32), 0, 0..matrix.len() as u32);
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
                cull_mode: Some(Face::Back), // this was NONE, was there a reason?
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
