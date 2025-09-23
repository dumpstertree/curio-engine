mod camera_rendering_components;
mod egui_app_state;
mod egui_tools;
mod render_feature_2d;
mod render_feature_2ds {
    pub mod render_feature_draw_ui;
}
mod render_feature_3d;
mod render_feature_3ds {
    pub mod render_feature_draw_gizmos;
    pub mod render_feature_draw_mesh;
}
use crate::render_feature_2d::RenderFeature2D;
use crate::render_feature_2ds::render_feature_draw_ui::RenderFeatureDrawUI;
use crate::render_feature_3d::RenderFeature3D;
// use crate::render_feature_3ds::render_feature_draw_gizmos::RenderFeatureDrawGizmo;
use crate::{camera_rendering_components::CameraRenderingComponents, egui_tools::EguiRenderer};
use built_in_state::state_camera::CameraState;
use core::collections::event_queue::EventQueue;
use core::collections::game_state::GameState;
use core::graphics::graphics_mapping::GraphicsMapping;
use core::io::texture_asset::TextureAsset;
use core::system::system_component::SystemComponent;
use core::system::system_components::system_component_graphics::SystemComponentGraphics;
use core::system_adapters::adapter_system_gpu::SystemGPU;
use egui_wgpu::wgpu::{BindGroup, CommandEncoder, DepthStencilState, Device, FragmentState, RenderPass, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPipeline, RenderPipelineDescriptor, Sampler, Surface, SurfaceTexture, Texture, TextureView};
use render_feature_3ds::render_feature_draw_mesh::RenderFeatureDrawMesh;
use std::{iter, vec};
use winit::event::WindowEvent;

pub struct SystemComponentDefaultGraphics {
    egui_renderer: EguiRenderer,
    graphics_mappings: Vec<GraphicsMapping>,
    render_features_3d: Vec<Box<dyn RenderFeature3D>>,
    render_features_2d: Vec<Box<dyn RenderFeature2D>>,
    is_dirty: bool,
    camera_rendering: CameraRenderingComponents,
    offscreen_texture: Texture,
    offscreen_view: TextureView,
    post_sampler: Sampler,
    post_pipeline: RenderPipeline,
    post_bind_group: BindGroup,
}

impl SystemComponent for SystemComponentDefaultGraphics {
    fn order(&self) -> i32 {
        9000
    }
    fn init(&mut self, _game_state: &mut Vec<GameState>) {}

    fn tick(&mut self, game_state: &mut Vec<GameState>, event_queue: &mut Vec<EventQueue>) {
        // system
        let surface = &SystemGPU::get_surface();
        let device = &SystemGPU::get_device();
        let queue = &SystemGPU::get_queue();

        // create the output
        let output = SystemComponentDefaultGraphics::get_output_texture(surface);
        let mut encoder = SystemComponentDefaultGraphics::get_encoder(device);
        {
            // draw all
            let offscreen_view = &mut self.offscreen_view;
            SystemComponentDefaultGraphics::draw_3d_features(&self.camera_rendering, &mut self.render_features_3d, &mut self.graphics_mappings, game_state, &mut encoder, offscreen_view);
            // self.draw_2d_features(game_state, &mut encoder, &self.offscreen_view, event_queue);
            SystemComponentDefaultGraphics::draw_post_process_features(&self.camera_rendering, &mut self.render_features_3d, &mut self.graphics_mappings, game_state, &mut encoder, offscreen_view);
        }

        {
            let depth = &SystemGPU::get_depth_texture();

            let mut rpass = encoder.begin_render_pass(&egui_wgpu::wgpu::RenderPassDescriptor {
                label: Some("postprocess pass"),
                color_attachments: &[Some(egui_wgpu::wgpu::RenderPassColorAttachment {
                    view: &output.texture.create_view(&Default::default()),
                    resolve_target: None,
                    ops: egui_wgpu::wgpu::Operations {
                        load: egui_wgpu::wgpu::LoadOp::Clear(egui_wgpu::wgpu::Color::WHITE),
                        store: egui_wgpu::wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: SystemComponentDefaultGraphics::get_depth_attatchment(depth),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rpass.set_pipeline(&self.post_pipeline);
            rpass.set_bind_group(0, &self.post_bind_group, &[]);
            rpass.draw(0..3, 0..1); // fullscreen triangle
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
    pub fn new() -> Box<SystemComponentDefaultGraphics> {
        let c = SystemGPU::get_config();
        let w = &(*SystemGPU::get_window());
        let d = &(*SystemGPU::get_device());

        let surface_config = SystemGPU::get_config();

        let device = SystemGPU::get_device();
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
        let post_sampler = sampler;

        let post_bind_group_layout = device.create_bind_group_layout(&egui_wgpu::wgpu::BindGroupLayoutDescriptor {
            label: Some("postprocess bind group layout"),
            entries: &[
                egui_wgpu::wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: egui_wgpu::wgpu::ShaderStages::FRAGMENT,
                    ty: egui_wgpu::wgpu::BindingType::Sampler(egui_wgpu::wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                egui_wgpu::wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: egui_wgpu::wgpu::ShaderStages::FRAGMENT,
                    ty: egui_wgpu::wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: egui_wgpu::wgpu::TextureViewDimension::D2,
                        sample_type: egui_wgpu::wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
            ],
        });

        let post_bind_group = device.create_bind_group(&egui_wgpu::wgpu::BindGroupDescriptor {
            label: Some("postprocess bind group"),
            layout: &post_bind_group_layout,
            entries: &[
                egui_wgpu::wgpu::BindGroupEntry {
                    binding: 0,
                    resource: egui_wgpu::wgpu::BindingResource::Sampler(&post_sampler),
                },
                egui_wgpu::wgpu::BindGroupEntry {
                    binding: 1,
                    resource: egui_wgpu::wgpu::BindingResource::TextureView(&offscreen_view),
                },
            ],
        });

        // --- Shader ---
        let shader = device.create_shader_module(egui_wgpu::wgpu::ShaderModuleDescriptor {
            label: Some("postprocess shader"),
            source: egui_wgpu::wgpu::ShaderSource::Wgsl(include_str!("postprocess.wgsl").into()),
        });

        // --- Pipeline layout ---
        let pipeline_layout = device.create_pipeline_layout(&egui_wgpu::wgpu::PipelineLayoutDescriptor {
            label: Some("postprocess pipeline layout"),
            bind_group_layouts: &[&post_bind_group_layout],
            push_constant_ranges: &[],
        });

        // --- Render pipeline ---
        let post_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("postprocess pipeline"),
            layout: Some(&pipeline_layout),
            vertex: egui_wgpu::wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_fullscreen"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_fullscreen"),
                targets: &[Some(egui_wgpu::wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(egui_wgpu::wgpu::BlendState::REPLACE),
                    write_mask: egui_wgpu::wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: egui_wgpu::wgpu::PrimitiveState::default(),
            depth_stencil: Some(egui_wgpu::wgpu::DepthStencilState {
                format: TextureAsset::DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: egui_wgpu::wgpu::CompareFunction::Less,
                stencil: egui_wgpu::wgpu::StencilState::default(),
                bias: egui_wgpu::wgpu::DepthBiasState::default(),
            }),
            multisample: egui_wgpu::wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Box::new(SystemComponentDefaultGraphics {
            egui_renderer: EguiRenderer::new(d, c.format, None, 1, w),
            graphics_mappings: Vec::new(),
            render_features_3d: vec![RenderFeatureDrawMesh::new()],
            render_features_2d: vec![RenderFeatureDrawUI::new()],
            is_dirty: true,
            camera_rendering: CameraRenderingComponents::new(1),
            offscreen_texture,
            offscreen_view,
            post_sampler,
            post_pipeline,
            post_bind_group,
        })
    }

    fn draw_post_process_features(
        camera_rendering: &CameraRenderingComponents,
        render_features_3d: &mut Vec<Box<dyn RenderFeature3D>>,
        graphics_mappings: &mut Vec<GraphicsMapping>,
        game_state: &mut Vec<GameState>,
        encoder: &mut egui_wgpu::wgpu::CommandEncoder,
        target_view: &mut egui_wgpu::wgpu::TextureView, // <-- changed from SurfaceTexture
    ) {
        let depth = &SystemGPU::get_depth_texture();

        let mut rpass = encoder.begin_render_pass(&egui_wgpu::wgpu::RenderPassDescriptor {
            label: Some("postprocess pass"),
            color_attachments: &[Some(egui_wgpu::wgpu::RenderPassColorAttachment {
                view: &output.texture.create_view(&Default::default()),
                resolve_target: None,
                ops: egui_wgpu::wgpu::Operations {
                    load: egui_wgpu::wgpu::LoadOp::Clear(egui_wgpu::wgpu::Color::WHITE),
                    store: egui_wgpu::wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: SystemComponentDefaultGraphics::get_depth_attatchment(depth),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        rpass.set_pipeline(&self.post_pipeline);
        rpass.set_bind_group(0, &self.post_bind_group, &[]);
        rpass.draw(0..3, 0..1); // fullscreen triangle
    }
    fn draw_3d_features(
        camera_rendering: &CameraRenderingComponents,
        render_features_3d: &mut Vec<Box<dyn RenderFeature3D>>,
        graphics_mappings: &mut Vec<GraphicsMapping>,
        game_state: &mut Vec<GameState>,
        encoder: &mut egui_wgpu::wgpu::CommandEncoder,
        target_view: &mut egui_wgpu::wgpu::TextureView, // <-- changed from SurfaceTexture
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
                feature.render(game_state, &mut render_pass, camera_rendering, i);
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
        let i = 2;
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
    fn get_render_pass<'a>(output: &'a SurfaceTexture, encoder: &'a mut CommandEncoder) -> RenderPass<'a> {
        let depth = &SystemGPU::get_depth_texture();

        let view = output
            .texture
            .create_view(&egui_wgpu::wgpu::TextureViewDescriptor::default());
        let render_pass = encoder.begin_render_pass(&egui_wgpu::wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[SystemComponentDefaultGraphics::get_color_atatchment(&view)],
            depth_stencil_attachment: SystemComponentDefaultGraphics::get_depth_attatchment(depth),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass
    }

    fn get_output_texture(surface: &Surface) -> SurfaceTexture {
        surface.get_current_texture().unwrap()
    }
    fn get_encoder(device: &Device) -> CommandEncoder {
        device.create_command_encoder(&egui_wgpu::wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") })
    }
    fn get_color_atatchment<'a>(view: &'a TextureView) -> Option<RenderPassColorAttachment<'a>> {
        Some(egui_wgpu::wgpu::RenderPassColorAttachment {
            view: view,
            resolve_target: None,
            ops: egui_wgpu::wgpu::Operations {
                load: egui_wgpu::wgpu::LoadOp::Clear(egui_wgpu::wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 }),
                store: egui_wgpu::wgpu::StoreOp::Store,
            },
        })
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
}
