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
use crate::egui_tools::EguiRenderer;
use crate::render_feature_2d::RenderFeature2D;
use crate::render_feature_2ds::render_feature_draw_ui::RenderFeatureDrawUI;
use crate::render_feature_3d::RenderFeature3D;
use crate::render_feature_3ds::render_feature_draw_gizmos::RenderFeatureDrawGizmo;
use built_in_state::state_camera::CameraState;
use core::collections::game_state::{self, GameState};
use core::collections::vector3::Vector3;
use core::collections::{camera_uniform::CameraSnapshot, event_queue::EventQueue};
use core::graphics::graphics_mapping::{self, GraphicsMapping};
use core::io::texture_asset::TextureAsset;
use core::system::system_component::SystemComponent;
use core::system::system_components::system_component_graphics::SystemComponentGraphics;
use core::system_adapters::adapter_system_gpu::SystemGPU;
use egui_wgpu::wgpu::{CommandEncoder, Device, RenderPass, RenderPassColorAttachment, RenderPassDepthStencilAttachment, Surface, SurfaceTexture, TextureView};
use render_feature_3ds::render_feature_draw_mesh::RenderFeatureDrawMesh;
use std::{iter, vec};
use winit::event::WindowEvent;

pub struct SystemComponentDefaultGraphics {
    egui_renderer: EguiRenderer,
    graphics_mappings: Vec<GraphicsMapping>,
    render_features_3d: Vec<Box<dyn RenderFeature3D>>,
    render_features_2d: Vec<Box<dyn RenderFeature2D>>,
    is_dirty: bool,
}

impl SystemComponent for SystemComponentDefaultGraphics {
    fn order(&self) -> i32 {
        9000
    }
    fn tick(&mut self, game_state: &mut Vec<GameState>, event_queue: &mut Vec<EventQueue>) {
        for i in 0..game_state.len() {
            let mut game_state = &mut game_state[i];
            let mut event_queue = &mut event_queue[i];

            // system
            let surface = &SystemGPU::get_surface();
            let device = &SystemGPU::get_device();
            let queue = &SystemGPU::get_queue();

            // create the output
            let output = SystemComponentDefaultGraphics::get_output_texture(surface);
            let mut encoder = SystemComponentDefaultGraphics::get_encoder(device);

            // draw all
            self.draw_3d_features(&mut game_state, &mut encoder, &output);
            self.draw_2d_features(&mut game_state, &mut encoder, &output, &mut event_queue);

            // submit commands for execution
            queue.submit(iter::once(encoder.finish()));
            // present the completed texture
            output.present();
        }
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

        Box::new(SystemComponentDefaultGraphics {
            egui_renderer: EguiRenderer::new(d, c.format, None, 1, w),
            graphics_mappings: Vec::new(),
            render_features_3d: vec![RenderFeatureDrawMesh::new(), RenderFeatureDrawGizmo::new()],
            render_features_2d: vec![RenderFeatureDrawUI::new()],
            is_dirty: true,
        })
    }
    fn draw_3d_features(&mut self, game_state: &mut GameState, encoder: &mut CommandEncoder, output: &SurfaceTexture) {
        // get system values
        let queue = &SystemGPU::get_queue();

        // get gamestate values
        let state_camera = game_state.get_value2::<CameraState>();

        // generate a render pass for this instance
        let mut render_pass = SystemComponentDefaultGraphics::get_render_pass(&output, encoder);

        // iterate over each camera in state
        for i in 0..(self.graphics_mappings.len() as usize) {
            // get camera data
            let cur_camera_snapshot = &state_camera.cameras;
            // for graphics_mapping in &self.graphics_mappings {
            let cur_graphics_mapping = &self.graphics_mappings[i];

            // create viewport bounds
            let size_x = state_camera.resolution_width as f32;
            let size_y = state_camera.resolution_height as f32;
            let x = cur_graphics_mapping.viewport_min.x * size_x;
            let y = cur_graphics_mapping.viewport_min.y * size_y;
            let w = ((cur_graphics_mapping.viewport_max.x - cur_graphics_mapping.viewport_min.x) * size_x).round();
            let h = ((cur_graphics_mapping.viewport_max.y - cur_graphics_mapping.viewport_min.y) * size_y).round();

            // set the viewport based on mapping
            render_pass.set_viewport(x, y, w, h, 0.0, 1.0);

            //calculate camera binding values
            let camera_uniform = cur_camera_snapshot.get_uniform(w as i32, h as i32);
            let camera_rendereing = camera_rendering_components::CameraRenderingComponents::new(camera_uniform);

            // write camera
            queue.write_buffer(&camera_rendereing.camera_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));

            // render
            for feature in self.render_features_3d.iter_mut() {
                feature.render(game_state, &mut render_pass, &camera_rendereing.camera_bind_group, &camera_rendereing.camera_bind_group_layout);
            }
        }
        // done using now clear all data
        for feature in self.render_features_3d.iter_mut() {
            feature.clear(game_state);
        }
    }
    fn draw_2d_features(&mut self, game_state: &mut GameState, encoder: &mut CommandEncoder, output: &SurfaceTexture, event_queue: &mut EventQueue) {
        for _ in 0..(self.graphics_mappings.len() as usize) {
            for feature in self.render_features_2d.iter_mut() {
                feature.render(game_state, event_queue, &output, encoder, &mut self.egui_renderer);
            }

            for feature in self.render_features_2d.iter_mut() {
                // feature.clear(game_state);
            }
        }
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
