use crate::system::system_component::ISystemComponent;
use crate::system::system_components::gameplay_components::gameplay_component_default::EngineCommands;
use crate::system::system_components::graphics_component::IGraphicsComponent;
use crate::system::system_game_states::state_camera::{CameraState, Projection};
use crate::system::system_game_states::state_draw::DrawCallsState;
use crate::system_adapters::adapter_system_gpu::{SystemGPU, SYS_GPU};
use crate::Collections::camera_uniform::CameraUniform;
use crate::Collections::game_state::GameState;
use crate::Collections::matrix4x4::Matrix4x4;
use crate::Collections::GraphicsBufferCache::Graphics_buffer_cache;
use crate::Collections::{DrawCall::DrawCall, Mesh::Vertex};
use crate::IO::texture_asset::Texture_asset;
use cgmath::Rad;
use std::iter;
use wgpu::{
    util::DeviceExt, Buffer, CommandEncoder, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPipeline, ShaderModule,
    SurfaceTexture, TextureView,
};
use wgpu::{BindGroup, BindGroupLayout, Device, Surface, SurfaceConfiguration};

pub struct WGPUGraphicsComponent {
    buffer_cache: Graphics_buffer_cache,
    camera_rendereing: CameraRenderingComponents,
    projection: Projection,
}

impl ISystemComponent for WGPUGraphicsComponent {
    fn order(&self) -> i32 {
        9000
    }
    fn init(&mut self, gs: &mut GameState) {
        self.camera_rendereing = CameraRenderingComponents::new(&self.projection);
    }
    fn render(&mut self, game_state: &mut GameState) -> &[EngineCommands] {
        let sys = SYS_GPU.lock().unwrap();

        let Some(window) = &sys.window else {
            return &[];
        };
        let Some(device) = &sys.device else {
            return &[];
        };
        let Some(queue) = &sys.queue else {
            return &[];
        };
        let Some(surface) = &sys.surface else {
            return &[];
        };
        let Some(config) = &sys.config else {
            return &[];
        };
        let Some(depth) = &sys.depth_texture else {
            return &[];
        };

        // get gamestate data
        let state_camera = game_state.get_value2::<CameraState>();
        let state_draws = game_state.get_value2::<DrawCallsState>();

        // redraw on window
        window.request_redraw();
        //
        let output = WGPUGraphicsComponent::get_output_texture(surface);
        let mut encoder = WGPUGraphicsComponent::get_encoder(device);
        {
            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[WGPUGraphicsComponent::get_color_atatchment(&view)],
                depth_stencil_attachment: WGPUGraphicsComponent::get_depth_attatchment(depth),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // update camera
            // camera.camera_uniform.update_view_proj(&state_camera);

            self.camera_rendereing
                .update_view_matrix(state_camera, &self.projection);

            // camera.update_view_matrix();
            queue.write_buffer(
                &self.camera_rendereing.camera_buffer,
                0,
                bytemuck::cast_slice(&[self.camera_rendereing.camera_uniform]),
            );

            // get all draw calls from state
            let draw_calls = &state_draws.draw_calls;
            for draw_call in draw_calls {
                // iterate over each mesh in the draw call
                for i in 0..draw_call.mesh.len() {
                    //
                    let mesh = &draw_call.mesh[i];
                    let material = &draw_call.materials[i];
                    // create material bind group
                    // let diffuse_bind_group = WGPUGraphicsComponent::get_diffuse_binding(&state, &material.textures[..]);
                    let diffuse_bind_group = material.get_binding_group(device);
                    // set render pipeline
                    let rp = WGPUGraphicsComponent::get_render_pipeline(
                        &self.camera_rendereing.camera_bind_group_layout,
                        config,
                        device,
                        material.shader.clone(),
                        &diffuse_bind_group.1,
                    );
                    render_pass.set_pipeline(&rp);

                    // create the instance buffer
                    let n_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Instance Buffer"),
                        contents: bytemuck::cast_slice(&draw_call.matrix),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
                    // fetch the cached buffers for the mesh
                    let buffers: (&Buffer, &Buffer) = self.buffer_cache.get_vertex_buffer(device, mesh);

                    // set buffers
                    render_pass.set_vertex_buffer(0, buffers.0.slice(..));
                    render_pass.set_vertex_buffer(1, n_buffer.slice(..));
                    render_pass.set_index_buffer(buffers.1.slice(..), wgpu::IndexFormat::Uint32);

                    // set bind groups
                    render_pass.set_bind_group(0, &diffuse_bind_group.0, &[]);
                    render_pass.set_bind_group(1, &self.camera_rendereing.camera_bind_group, &[]);
                    // draw
                    render_pass.draw_indexed(0..(mesh.indicies.len() as u32), 0, 0..draw_call.matrix.len() as u32);
                }
            }
        }
        // submit commands for execution
        queue.submit(iter::once(encoder.finish()));
        // present the completed texture
        output.present();

        return &[];
        // return a success
    }
}
impl IGraphicsComponent for WGPUGraphicsComponent {}
impl WGPUGraphicsComponent {
    pub fn new() -> WGPUGraphicsComponent {
        let p = Projection::new(1920, 1080, cgmath::Deg(45.0), 0.1, 100.0);
        WGPUGraphicsComponent {
            buffer_cache: Graphics_buffer_cache::new(),
            camera_rendereing: CameraRenderingComponents::new(&p),
            projection: p,
        }
    }

    fn get_render_pipeline(
        camera_bind: &BindGroupLayout,
        config: &SurfaceConfiguration,
        device: &Device,
        shader: ShaderModule,
        texture_bind_group_layout: &BindGroupLayout,
    ) -> RenderPipeline {
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &camera_bind],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                // buffers: &[super::model::ModelVertex::desc(), InstanceRaw::desc()],
                buffers: &[Vertex::desc(), Matrix4x4::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture_asset::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // 1.
                stencil: wgpu::StencilState::default(),     // 2.
                bias: wgpu::DepthBiasState::default(),
            }),
            // depth_stencil: None,
            multisample: wgpu::MultisampleState {
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

    fn get_output_texture(surface: &Surface) -> SurfaceTexture {
        surface.get_current_texture().unwrap()
    }
    fn get_encoder(device: &Device) -> CommandEncoder {
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        })
    }
    fn get_color_atatchment<'a>(view: &'a TextureView) -> Option<RenderPassColorAttachment<'a>> {
        Some(wgpu::RenderPassColorAttachment {
            view: view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.1,
                    g: 0.2,
                    b: 0.3,
                    a: 1.0,
                }),
                store: wgpu::StoreOp::Store,
            },
        })
    }
    fn get_depth_attatchment<'a>(depth: &'a Texture_asset) -> Option<RenderPassDepthStencilAttachment<'a>> {
        Some(wgpu::RenderPassDepthStencilAttachment {
            view: &depth.view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        })
    }
}

// #[derive(Clone)]
// pub struct DrawCallsState {
//     pub draw_calls: Vec<DrawCall>,
// }
// impl DrawCallsState {
//     pub fn new<'a>() -> DrawCallsState {
//         DrawCallsState { draw_calls: Vec::new() }
//     }
// }

pub struct CameraRenderingComponents {
    camera_bind_group: BindGroup,
    camera_bind_group_layout: BindGroupLayout,
    camera_uniform: CameraUniform,
    camera_buffer: Buffer,
}
impl CameraRenderingComponents {
    pub fn update_view_matrix(&mut self, state_camera: CameraState, projection: &Projection) {
        self.camera_uniform
            .update_view_proj(&state_camera, &projection)
    }

    pub fn new(projection: &Projection) -> CameraRenderingComponents {
        let sys = SYS_GPU.lock().unwrap();

        let Some(device) = &sys.device else { panic!("Panic") };
        let camera = CameraState::default();
        let mut camera_uniform: CameraUniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera, projection);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        CameraRenderingComponents {
            camera_bind_group,
            camera_bind_group_layout,
            camera_uniform,
            camera_buffer,
        }
    }
}
