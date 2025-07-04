use crate::game_state::GameState;
use crate::system::system_component::ISystemComponent;
use crate::system::system_components::graphics_component::IGraphicsComponent;
use crate::system_adapters::adapter_system_gpu::SYS_GPU;
use crate::texture;
use crate::Collections::matrix4x4::Matrix4x4;
use crate::Collections::GraphicsBufferCache::Graphics_buffer_cache;
use crate::Window::CameraState;
use crate::{
    Collections::{DrawCall::DrawCall, Mesh::Vertex},
    Window::state::State,
    IO::AssetLoader::AssetLoader,
};
use cgmath::{Quaternion, Vector3};
use serde::de;
use std::iter;
use wgpu::wgc::device::queue;
use wgpu::{
    util::DeviceExt, Buffer, CommandEncoder, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPipeline, ShaderModule,
    SurfaceTexture, TextureView,
};
use wgpu::{BindGroupLayout, Device, Surface};

pub struct WGPUGraphicsComponent {
    buffer_cache: Graphics_buffer_cache,
    count: i32,
}

impl ISystemComponent for WGPUGraphicsComponent {
    fn order(&self) -> i32 {
        9000
    }
    fn init(&mut self, state: &mut State, gs: &mut GameState) {
        println!("init graphics");
    }
    fn render(&mut self, state: &mut State, game_state: &mut GameState) {
        let sys = SYS_GPU.lock().unwrap();
        let Some(device) = &sys.device else {
            return;
        };
        let Some(queue) = &sys.queue else {
            return;
        };
        let Some(window) = &sys.window else {
            return;
        };
        let Some(surface) = &sys.surface else {
            panic!();
        };
        // We can't render unless the surface is configured
        if !state.is_surface_configured {
            return;
        }

        // get gamestate data
        let state_camera = game_state.get_camera();
        let state_draws = game_state.get_draw_calls();

        // redraw on window
        window.request_redraw();
        //
        let output = WGPUGraphicsComponent::get_output_texture(surface);
        let mut encoder = WGPUGraphicsComponent::get_encoder(device);
        {
            let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[WGPUGraphicsComponent::get_color_atatchment(&view)],
                depth_stencil_attachment: WGPUGraphicsComponent::get_depth_attatchment(state),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // update camera
            state.camera_uniform.update_view_proj(&state_camera);
            queue.write_buffer(&state.camera_buffer, 0, bytemuck::cast_slice(&[state.camera_uniform]));
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
                    let rp = WGPUGraphicsComponent::get_render_pipeline(device, &state, material.shader.clone(), &diffuse_bind_group.1);
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
                    render_pass.set_bind_group(1, &state.camera_bind_group, &[]);
                    // draw
                    render_pass.draw_indexed(0..(mesh.indicies.len() as u32), 0, 0..draw_call.matrix.len() as u32);
                }
            }
        }
        // submit commands for execution
        queue.submit(iter::once(encoder.finish()));
        // present the completed texture
        output.present();
        self.count = self.count + 1;
        // return a success
    }
}

impl WGPUGraphicsComponent {}
impl IGraphicsComponent for WGPUGraphicsComponent {}

pub const KEY_CAMERA_STATE: i32 = 0;
pub const KEY_APPLICATION_STATE: i32 = 1;
pub const KEY_DRAW_CALLS_STATE: i32 = 2;

impl WGPUGraphicsComponent {
    pub fn new() -> WGPUGraphicsComponent {
        WGPUGraphicsComponent {
            buffer_cache: Graphics_buffer_cache::new(),
            count: 0,
        }
    }

    fn get_render_pipeline(device: &Device, state: &State, shader: ShaderModule, texture_bind_group_layout: &BindGroupLayout) -> RenderPipeline {
        // let texture_bind_group_layout = state.box_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //     entries: &[
        //         wgpu::BindGroupLayoutEntry {
        //             binding: 0,
        //             visibility: wgpu::ShaderStages::FRAGMENT,
        //             ty: wgpu::BindingType::Texture {
        //                 multisampled: false,
        //                 view_dimension: wgpu::TextureViewDimension::D2,
        //                 sample_type: wgpu::TextureSampleType::Float { filterable: true },
        //             },
        //             count: None,
        //         },
        //         wgpu::BindGroupLayoutEntry {
        //             binding: 1,
        //             visibility: wgpu::ShaderStages::FRAGMENT,
        //             ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        //             count: None,
        //         },
        //     ],
        //     label: Some("texture_bind_group_layout"),
        // });
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
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
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
                    format: state.config.format,
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
                format: texture::Texture::DEPTH_FORMAT,
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
    fn get_depth_attatchment<'a>(state: &'a State) -> Option<RenderPassDepthStencilAttachment<'a>> {
        Some(wgpu::RenderPassDepthStencilAttachment {
            view: &state.depth_texture.view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        })
    }
}

#[derive(Clone)]
pub struct DrawCallsState {
    pub draw_calls: Vec<DrawCall>,
}
impl DrawCallsState {
    pub fn new<'a>() -> DrawCallsState {
        DrawCallsState { draw_calls: Vec::new() }
    }
    pub fn new2<'a>(drawcalls: Vec<DrawCall>) -> DrawCallsState {
        DrawCallsState { draw_calls: drawcalls }
    }
}
