mod camera_rendering_components;
mod egui_app_state;
mod egui_tools;
mod render_feature_2d;
mod shadow_camera;
mod shadow_system;
mod render_feature_2ds {
    pub mod render_feature_draw_ui;
}
mod render_feature_3d;
mod render_feature_3ds {
    pub mod render_feature_draw_gizmos;
    pub mod render_feature_draw_mesh;
}
mod render_feature_post_process;
mod render_feature_post_processes {
    pub mod render_feature_post_process_fog;
    pub mod render_feature_post_process_kuwahara;
    pub mod render_feature_post_process_sobel_outline;
}
use crate::render_feature_2d::RenderFeature2D;
use crate::render_feature_2ds::render_feature_draw_ui::RenderFeatureDrawUI;
use crate::render_feature_3d::RenderFeature3D;
use crate::render_feature_post_process::RenderFeaturePostProcess;
use crate::render_feature_post_processes::render_feature_post_process_fog::RenderFeaturePostProcessFog;
use crate::render_feature_post_processes::render_feature_post_process_kuwahara::RenderFeaturePostProcessKuwahara;
use crate::render_feature_post_processes::render_feature_post_process_sobel_outline::RenderFeaturePostProcessOutline;
use crate::shadow_system::ShadowSystem;
// use crate::render_feature_post_processes::render_feature_post_process_kuwahara::RenderFeaturePostProcessKuwahara;
// use crate::render_feature_3ds::render_feature_draw_gizmos::RenderFeatureDrawGizmo;
use crate::{camera_rendering_components::CameraRenderingComponents, egui_tools::EguiRenderer};
use built_in_state::state_camera::CameraState;
use built_in_state::state_draw::DrawCallsState;
use built_in_state::state_lights::StateLights;
use built_in_state::state_time::TimeState;
use core::collections::event_queue::EventQueue;
use core::collections::game_state::{self, GameState};
use core::collections::light_uniform::{DrawCallLight, LightSystem};
use core::collections::matrix4x4::{Matrix4x4, QuadVertex};
use core::collections::quaternion::Quaternion;
use core::collections::vector3::Vector3;
use core::graphics::graphics_mapping::GraphicsMapping;
use core::io::texture_asset::TextureAsset;
use core::system::system_component::SystemComponent;
use core::system::system_components::system_component_graphics::SystemComponentGraphics;
use core::system_adapters::adapter_system_gpu::SystemGPU;
use egui_wgpu::wgpu::util::DeviceExt;
use egui_wgpu::wgpu::{BindGroup, CommandEncoder, DepthStencilState, Device, FragmentState, RenderPass, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPipeline, RenderPipelineDescriptor, Sampler, Surface, SurfaceTexture, Texture, TextureView};
use render_feature_3ds::render_feature_draw_mesh::RenderFeatureDrawMesh;
use std::sync::Arc;
use std::{iter, vec};
use winit::event::WindowEvent;

pub struct SystemComponentDefaultGraphics {
    egui_renderer: EguiRenderer,
    graphics_mappings: Vec<GraphicsMapping>,
    render_features_post_process: Vec<Box<dyn RenderFeaturePostProcess>>,
    render_features_3d: Vec<Box<dyn RenderFeature3D>>,
    render_features_2d: Vec<Box<dyn RenderFeature2D>>,
    is_dirty: bool,
    camera_rendering: CameraRenderingComponents,
    offscreen_view: TextureView,
    post_process_resources: PostProcessResources,
    shadow_system: ShadowSystem,
    shadow_debug_pipeline: Option<egui_wgpu::wgpu::RenderPipeline>,
    shadow_debug_bind_group: Option<egui_wgpu::wgpu::BindGroup>,
    shadow_debug_quad: Option<(egui_wgpu::wgpu::Buffer, egui_wgpu::wgpu::Buffer)>,
}

impl SystemComponent for SystemComponentDefaultGraphics {
    fn order(&self) -> i32 {
        9000
    }
    fn init(&mut self, _game_state: &mut Vec<GameState>) {
        let device = &SystemGPU::get_device();

        let config = &SystemGPU::get_config();

        self.init_shadow_debug(&device);
        let debug_shader = device.create_shader_module(egui_wgpu::wgpu::ShaderModuleDescriptor {
            label: Some("shadow debug shader"),
            source: egui_wgpu::wgpu::ShaderSource::Wgsl(include_str!("shadow_debug.wgsl").into()),
        });
        let x = SystemComponentDefaultGraphics::create_shadow_debug_pipeline(&device, &debug_shader, &self.shadow_system, config.format);
        self.shadow_debug_bind_group = x.1;
        self.shadow_debug_pipeline = x.0;
    }

    fn tick(&mut self, game_state: &mut Vec<GameState>, event_queue: &mut Vec<EventQueue>) {
        // system
        let surface = &SystemGPU::get_surface();
        let device = &SystemGPU::get_device();
        let queue = &SystemGPU::get_queue();

        // create the output
        let output = SystemComponentDefaultGraphics::get_output_texture(surface);
        let mut encoder = SystemComponentDefaultGraphics::get_encoder(device);

        // for game_state in game_state.iter_mut() {
        if game_state[0].get_value2::<StateLights>().all_lights.len() > 0 {
            // let light = &game_state[0].get_value2::<StateLights>().all_lights[0];
            // let light_pos = Vector3::new(light.position[0], light.position[1], light.position[2]);
            // let light_rot = Quaternion::from_look_rotation(Vector3::new(light.direction[0], light.direction[1], light.direction[2]), Vector3::up());
            // let matrix = Matrix4x4::new(light_pos, light_rot, Vector3::one());
            self.shadow_system
                .update(game_state[0].get_value2::<TimeState>().scaled_time as f32);
            self.shadow_system
                .render(&mut encoder, &game_state[0].get_value2::<DrawCallsState>().draw_calls);
        }
        // }
        // let lights = SystemComponentDefaultGraphics::collect_lights(game_state);
        // draw all 3d
        {
            // draw 3D into offscreen
            // let offscreen_view = &mut self.offscreen_view;
            // SystemComponentDefaultGraphics::draw_3d_features(&self.camera_rendering, &mut self.render_features_3d, &mut self.graphics_mappings, game_state, &mut encoder, offscreen_view, &self.shadow_system);
            if let (Some(pipeline), Some(bind_group), Some((vb, ib))) = (&self.shadow_debug_pipeline, &self.shadow_debug_bind_group, &self.shadow_debug_quad) {
                let mut pass = encoder.begin_render_pass(&egui_wgpu::wgpu::RenderPassDescriptor {
                    label: Some("Shadow Map Debug Pass"),
                    color_attachments: &[Some(egui_wgpu::wgpu::RenderPassColorAttachment {
                        view: &self.offscreen_view, // or swapchain view for direct output
                        resolve_target: None,
                        ops: egui_wgpu::wgpu::Operations {
                            load: egui_wgpu::wgpu::LoadOp::Clear(egui_wgpu::wgpu::Color::BLACK),
                            store: egui_wgpu::wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                pass.set_pipeline(pipeline);
                pass.set_bind_group(0, bind_group, &[]);
                pass.set_vertex_buffer(0, vb.slice(..));
                pass.set_index_buffer(ib.slice(..), egui_wgpu::wgpu::IndexFormat::Uint16);
                pass.draw_indexed(0..6, 0, 0..1);
            }
            for game_state in game_state.iter_mut() {
                for x in &mut self.render_features_3d {
                    x.clear(game_state);
                }
            }
        }

        // draw all post-process
        {
            // post-processing into swapchain output
            SystemComponentDefaultGraphics::draw_post_features(
                &mut encoder,
                &mut self.render_features_post_process, // <- Vec<Box<dyn RenderFeaturePost>>
                &self.offscreen_view,                   // <- input: result of 3D rendering
                &self.post_process_resources,           // <- contains texture_a/view_a, texture_b/view_b
                &output
                    .texture
                    .create_view(&egui_wgpu::wgpu::TextureViewDescriptor::default()),
            );
        }
        // draw all 2d
        {
            self.draw_2d_features(game_state, &mut encoder, &output, event_queue);
        }

        // submit commands for execution
        queue.submit(iter::once(encoder.finish()));

        // present the completed texture
        output.present();
    }

    fn raw_event(&mut self, event: WindowEvent) {
        let window = SystemGPU::get_window();
        self.egui_renderer.handle_input(&window, &event);
    }
    fn set_game_mode(&mut self, game_state: &mut Vec<GameState>, game_mode: &core::dumpster_engine::GameMode) {
        let mut graphics_mapping = vec![];
        for x in &game_mode.game_instances {
            graphics_mapping.push(x.graphics_mappings.clone());
        }

        self.camera_rendering = CameraRenderingComponents::new(graphics_mapping.len());
        self.graphics_mappings = graphics_mapping;
        self.is_dirty = true;
    }
}
impl SystemComponentGraphics for SystemComponentDefaultGraphics {}
impl SystemComponentDefaultGraphics {
    // construction
    pub fn new() -> Box<SystemComponentDefaultGraphics> {
        let c = SystemGPU::get_config();
        let w = &(*SystemGPU::get_window());
        let d = &(*SystemGPU::get_device());

        let surface_config = SystemGPU::get_config();
        let device = SystemGPU::get_device();

        // --- Offscreen texture ---
        let offscreen_texture = device.create_texture(&egui_wgpu::wgpu::TextureDescriptor {
            label: Some("offscreen color"),
            size: egui_wgpu::wgpu::Extent3d {
                width: surface_config.width,
                height: surface_config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: egui_wgpu::wgpu::TextureDimension::D2,
            format: surface_config.format,
            usage: egui_wgpu::wgpu::TextureUsages::RENDER_ATTACHMENT | egui_wgpu::wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let offscreen_view = offscreen_texture.create_view(&Default::default());

        // --- Sampler ---
        let sampler = device.create_sampler(&egui_wgpu::wgpu::SamplerDescriptor {
            mag_filter: egui_wgpu::wgpu::FilterMode::Linear,
            min_filter: egui_wgpu::wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let depth_texture = SystemGPU::get_depth_texture();
        let depth_view = &depth_texture.view;

        let r = PostProcessResources::new(d, c.width, c.height, c.format);
        Box::new(SystemComponentDefaultGraphics {
            egui_renderer: EguiRenderer::new(d, c.format, None, 1, w),
            graphics_mappings: Vec::new(),
            render_features_3d: vec![RenderFeatureDrawMesh::new()],
            render_features_2d: vec![RenderFeatureDrawUI::new()],
            render_features_post_process: vec![
                RenderFeaturePostProcessKuwahara::new(device.clone(), c.format, &r, depth_view, &offscreen_view),
                RenderFeaturePostProcessOutline::new(device.clone(), c.format, &r, depth_view, &offscreen_view),
                // RenderFeaturePostProcessFog::new(device.clone(), c.format, &r, depth_view, &offscreen_view),
            ],
            is_dirty: true,
            camera_rendering: CameraRenderingComponents::new(1),
            offscreen_view,
            post_process_resources: r,
            shadow_system: ShadowSystem::new(Matrix4x4::default()),
            shadow_debug_pipeline: None,
            shadow_debug_bind_group: None,
            shadow_debug_quad: None,
        })
    }
    fn create_shadow_debug_pipeline(device: &egui_wgpu::wgpu::Device, shader: &egui_wgpu::wgpu::ShaderModule, shadow_system: &ShadowSystem, format: egui_wgpu::wgpu::TextureFormat) -> (Option<RenderPipeline>, Option<BindGroup>) {
        // Bind group layout
        let bind_layout = device.create_bind_group_layout(&egui_wgpu::wgpu::BindGroupLayoutDescriptor {
            label: Some("shadow debug layout"),
            entries: &[
                egui_wgpu::wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: egui_wgpu::wgpu::ShaderStages::FRAGMENT,
                    ty: egui_wgpu::wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: egui_wgpu::wgpu::TextureViewDimension::D2,
                        sample_type: egui_wgpu::wgpu::TextureSampleType::Depth,
                    },
                    count: None,
                },
                egui_wgpu::wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: egui_wgpu::wgpu::ShaderStages::FRAGMENT,
                    ty: egui_wgpu::wgpu::BindingType::Sampler(egui_wgpu::wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
            ],
        });

        // Bind group
        let bind_group = device.create_bind_group(&egui_wgpu::wgpu::BindGroupDescriptor {
            label: Some("shadow debug bind group"),
            layout: &bind_layout,
            entries: &[
                egui_wgpu::wgpu::BindGroupEntry {
                    binding: 0,
                    resource: egui_wgpu::wgpu::BindingResource::TextureView(&shadow_system.depth_view),
                },
                egui_wgpu::wgpu::BindGroupEntry {
                    binding: 1,
                    resource: egui_wgpu::wgpu::BindingResource::Sampler(&shadow_system.sampler),
                },
            ],
        });

        // Pipeline
        let pipeline_layout = device.create_pipeline_layout(&egui_wgpu::wgpu::PipelineLayoutDescriptor {
            label: Some("Shadow Debug Pipeline Layout"),
            bind_group_layouts: &[&bind_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&egui_wgpu::wgpu::RenderPipelineDescriptor {
            label: Some("Shadow Debug Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: egui_wgpu::wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &[QuadVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(egui_wgpu::wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_main"),
                targets: &[Some(egui_wgpu::wgpu::ColorTargetState {
                    format,
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

        (Some(pipeline), Some(bind_group))
    }
    // pub fn collect_lights(game_state: &Vec<GameState>) -> Vec<Light> {
    //     let mut out = Vec::new();
    //     for gs in game_state.iter() {
    //         // adapt to your component storage: here's an example
    //         let state_lights = gs.get_value2::<StateLights>()
    //         out.extend_from_slice(ls);
    //     }
    //     out
    // }
    // draw
    pub fn draw_post_features(
        encoder: &mut egui_wgpu::wgpu::CommandEncoder,
        post_features: &mut [Box<dyn RenderFeaturePostProcess>],
        input_view: &egui_wgpu::wgpu::TextureView, // offscreen scene render
        resources: &PostProcessResources,
        output_view: &egui_wgpu::wgpu::TextureView, // final swapchain target
    ) {
        use crate::render_feature_post_process::PostProcessSource;

        // src starts as the offscreen scene
        let mut src: &egui_wgpu::wgpu::TextureView = input_view;

        // ping/pong targets — keep them distinct and swap between them
        let mut ping: &egui_wgpu::wgpu::TextureView = &resources.view_a;
        let mut pong: &egui_wgpu::wgpu::TextureView = &resources.view_b;

        // track logical source for bind-group selection
        let mut current_source = PostProcessSource::Offscreen;

        let post_features_len = post_features.len();

        for (i, feature) in post_features.iter_mut().enumerate() {
            let is_last = i == post_features_len - 1;

            // destination: final swapchain if last pass, otherwise ping
            let target: &egui_wgpu::wgpu::TextureView = if is_last { output_view } else { ping };

            // run the pass
            feature.render(encoder, src, target, current_source);

            if !is_last {
                // advance the pipeline:
                // next src is what we just wrote (target)
                src = target;

                // advance logical source for bind-group selection
                current_source = match current_source {
                    PostProcessSource::Offscreen => PostProcessSource::ViewA,
                    PostProcessSource::ViewA => PostProcessSource::ViewB,
                    PostProcessSource::ViewB => PostProcessSource::ViewA,
                };

                // swap ping/pong so next write goes into the other ping-pong texture
                std::mem::swap(&mut ping, &mut pong);
            }
        }
    }

    fn draw_3d_features(
        camera_rendering: &CameraRenderingComponents,
        render_features_3d: &mut Vec<Box<dyn RenderFeature3D>>,
        graphics_mappings: &mut Vec<GraphicsMapping>,
        game_state: &mut Vec<GameState>,
        encoder: &mut egui_wgpu::wgpu::CommandEncoder,
        target_view: &mut egui_wgpu::wgpu::TextureView, // <-- changed from SurfaceTexture
        shadow_system: &ShadowSystem,
    ) {
        // generate a render pass for this instance
        let depth = SystemGPU::get_depth_texture();
        let mut render_pass = encoder.begin_render_pass(&egui_wgpu::wgpu::RenderPassDescriptor {
            label: Some("3D render pass"),
            color_attachments: &[Some(egui_wgpu::wgpu::RenderPassColorAttachment {
                view: target_view, // <-- use the texture view
                resolve_target: None,
                ops: egui_wgpu::wgpu::Operations {
                    load: egui_wgpu::wgpu::LoadOp::Clear(egui_wgpu::wgpu::Color { r: 0.1, g: 0.1, b: 0.1, a: 1.0 }),
                    store: egui_wgpu::wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: SystemComponentDefaultGraphics::get_depth_attatchment(&depth),
            timestamp_writes: None,
            occlusion_query_set: None, // keep or add depth if you use it
        });

        // iterate over each camera in state
        for i in 0..graphics_mappings.len() {
            let game_state = game_state.get_mut(i).unwrap();
            let state_camera = game_state.get_value2::<CameraState>();

            // get camera data
            let cur_camera_snapshot = &state_camera.cameras;
            let cur_graphics_mapping = &graphics_mappings[i];

            // create viewport bounds
            let size_x = state_camera.resolution_width as f32;
            let size_y = state_camera.resolution_height as f32;
            let x = cur_graphics_mapping.viewport_min.x * size_x;
            let y = cur_graphics_mapping.viewport_min.y * size_y;
            let w = ((cur_graphics_mapping.viewport_max.x - cur_graphics_mapping.viewport_min.x) * size_x).round();
            let h = ((cur_graphics_mapping.viewport_max.y - cur_graphics_mapping.viewport_min.y) * size_y).round();

            // set the viewport based on mapping
            render_pass.set_viewport(x, y, w, h, 0.0, 1.0);

            // calculate camera binding values
            let camera_uniform = cur_camera_snapshot.get_uniform(w as i32, h as i32);
            camera_rendering.update(i, &camera_uniform);
            let camera_rendering = &camera_rendering;

            // render features
            for feature in render_features_3d.iter_mut() {
                // render
                feature.render(game_state, &mut render_pass, camera_rendering, i, shadow_system);
            }
        }

        // cleanup
        for feature in render_features_3d.iter_mut() {
            for game_state in &mut *game_state {
                feature.clear(game_state);
            }
        }
    }
    fn draw_2d_features(&mut self, game_state: &mut Vec<GameState>, encoder: &mut CommandEncoder, output: &SurfaceTexture, event_queue: &mut Vec<EventQueue>) {
        // THIS IS HACKED BECAUSE WE CANT ALL WRITE TO THE MAIN SCREEN

        // for i in 0..(self.graphics_mappings.len() as usize) {
        let i = self.graphics_mappings.len() - 1;
        let game_state = game_state.get_mut(i).unwrap();
        let event_queue = event_queue.get_mut(i).unwrap();
        for feature in self.render_features_2d.iter_mut() {
            feature.render(game_state, event_queue, &output, encoder, &mut self.egui_renderer);
        }

        for feature in self.render_features_2d.iter_mut() {
            feature.clear(game_state);
        }
        // }
    }

    // get
    fn get_output_texture(surface: &Surface) -> SurfaceTexture {
        surface.get_current_texture().unwrap()
    }
    fn get_encoder(device: &Device) -> CommandEncoder {
        device.create_command_encoder(&egui_wgpu::wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") })
    }
    fn get_depth_attatchment<'a>(depth: &'a TextureAsset) -> Option<RenderPassDepthStencilAttachment<'a>> {
        Some(egui_wgpu::wgpu::RenderPassDepthStencilAttachment {
            view: &depth.view,
            depth_ops: Some(egui_wgpu::wgpu::Operations {
                load: egui_wgpu::wgpu::LoadOp::Clear(1.0),
                store: egui_wgpu::wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        })
    }

    fn init_shadow_debug(&mut self, device: &egui_wgpu::wgpu::Device) {
        let vertices = [QuadVertex::new([-1.0, -1.0], [0.0, 0.0]), QuadVertex::new([1.0, -1.0], [1.0, 0.0]), QuadVertex::new([1.0, 1.0], [1.0, 1.0]), QuadVertex::new([-1.0, 1.0], [0.0, 1.0])];

        let indices: [u16; 6] = [0, 1, 2, 2, 3, 0];

        let vertex_buffer = device.create_buffer_init(&egui_wgpu::wgpu::util::BufferInitDescriptor {
            label: Some("Shadow Debug Quad VB"),
            contents: bytemuck::cast_slice(&vertices),
            usage: egui_wgpu::wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&egui_wgpu::wgpu::util::BufferInitDescriptor {
            label: Some("Shadow Debug Quad IB"),
            contents: bytemuck::cast_slice(&indices),
            usage: egui_wgpu::wgpu::BufferUsages::INDEX,
        });

        self.shadow_debug_quad = Some((vertex_buffer, index_buffer));
    }
}

pub struct PostProcessResources {
    pub texture_a: egui_wgpu::wgpu::Texture,
    pub view_a: egui_wgpu::wgpu::TextureView,
    pub texture_b: egui_wgpu::wgpu::Texture,
    pub view_b: egui_wgpu::wgpu::TextureView,
}

impl PostProcessResources {
    pub fn new(device: &egui_wgpu::wgpu::Device, width: u32, height: u32, format: egui_wgpu::wgpu::TextureFormat) -> Self {
        let usage = egui_wgpu::wgpu::TextureUsages::RENDER_ATTACHMENT | egui_wgpu::wgpu::TextureUsages::TEXTURE_BINDING;
        let desc = |label| egui_wgpu::wgpu::TextureDescriptor {
            label: Some(label),
            size: egui_wgpu::wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: egui_wgpu::wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        };

        let texture_a = device.create_texture(&desc("post A"));
        let view_a = texture_a.create_view(&egui_wgpu::wgpu::TextureViewDescriptor::default());

        let texture_b = device.create_texture(&desc("post B"));
        let view_b = texture_b.create_view(&egui_wgpu::wgpu::TextureViewDescriptor::default());

        Self { texture_a, view_a, texture_b, view_b }
    }
}
