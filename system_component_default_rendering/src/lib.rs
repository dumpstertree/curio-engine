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
// use crate::render_feature_post_processes::render_feature_post_process_kuwahara::RenderFeaturePostProcessKuwahara;
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
    render_features_post_process: Vec<Box<dyn RenderFeaturePostProcess>>,
    render_features_3d: Vec<Box<dyn RenderFeature3D>>,
    render_features_2d: Vec<Box<dyn RenderFeature2D>>,
    is_dirty: bool,
    camera_rendering: CameraRenderingComponents,
    offscreen_view: TextureView,
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

        // draw all 3d
        {
            // draw 3D into offscreen
            let offscreen_view = &mut self.offscreen_view;
            SystemComponentDefaultGraphics::draw_3d_features(&self.camera_rendering, &mut self.render_features_3d, &mut self.graphics_mappings, game_state, &mut encoder, offscreen_view);
        }

        // draw all post-process
        {
            // post-processing into swapchain output
            let output_view = &output
                .texture
                .create_view(&egui_wgpu::wgpu::TextureViewDescriptor::default());
            SystemComponentDefaultGraphics::draw_post_features(
                &mut self.render_features_post_process,
                game_state,
                &mut encoder,
                &self.offscreen_view, // input
                &output_view,         // target
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
        let post_sampler = sampler;
        let depth_texture = SystemGPU::get_depth_texture();
        let depth_view = &depth_texture.view;

        Box::new(SystemComponentDefaultGraphics {
            egui_renderer: EguiRenderer::new(d, c.format, None, 1, w),
            graphics_mappings: Vec::new(),
            render_features_3d: vec![RenderFeatureDrawMesh::new()],
            render_features_2d: vec![RenderFeatureDrawUI::new()],
            render_features_post_process: vec![
                RenderFeaturePostProcessKuwahara::new(device.clone(), &offscreen_view, &post_sampler, depth_view),
                RenderFeaturePostProcessOutline::new(device.clone(), &offscreen_view, &post_sampler, depth_view),
                RenderFeaturePostProcessFog::new(device.clone(), &offscreen_view, &post_sampler, depth_view),
            ],
            is_dirty: true,
            camera_rendering: CameraRenderingComponents::new(1),
            offscreen_view,
        })
    }

    // draw
    fn draw_post_features(
        render_features_post: &mut Vec<Box<dyn RenderFeaturePostProcess>>,
        game_state: &mut Vec<GameState>,
        encoder: &mut egui_wgpu::wgpu::CommandEncoder,
        input_view: &egui_wgpu::wgpu::TextureView,  // ← offscreen (3D pass result)
        target_view: &egui_wgpu::wgpu::TextureView, // ← swapchain output
    ) {
        // new render pass for post-processing
        let mut render_pass = encoder.begin_render_pass(&egui_wgpu::wgpu::RenderPassDescriptor {
            label: Some("Post-processing render pass"),
            color_attachments: &[Some(egui_wgpu::wgpu::RenderPassColorAttachment {
                view: target_view,
                resolve_target: None,
                ops: egui_wgpu::wgpu::Operations {
                    load: egui_wgpu::wgpu::LoadOp::Load,
                    store: egui_wgpu::wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None, // post passes don’t need depth
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // iterate over features
        for feature in render_features_post.iter_mut() {
            // usually just one "screen" target, but keeping consistent with 3D pattern
            let game_state = game_state.get_mut(0).unwrap();
            feature.render(game_state, &mut render_pass, input_view);
        }

        // cleanup
        for feature in render_features_post.iter_mut() {
            for game_state in &mut *game_state {
                feature.clear(game_state);
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
}
