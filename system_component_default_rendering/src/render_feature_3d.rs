use built_in_state::state_camera::CameraState;
use core::{
    collections::{game_state::GameState, vector2::Vector2},
    graphics::graphics_mapping::GraphicsMapping,
    io::texture_asset::TextureAsset,
    system_adapters::adapter_system_gpu::SystemGPU,
};
use egui_wgpu::wgpu::{BindGroup, BindGroupLayout, RenderPass, RenderPassDepthStencilAttachment};

use crate::{camera_rendering_components::CameraRenderingComponents, render_feature_3ds::render_feature_draw_mesh::RenderFeatureDrawMesh, shadow_system::ShadowSystem};

pub trait RenderFeature3D {
    fn render(&mut self, game_state: &mut GameState, render_pass: &mut RenderPass, camera: &CameraRenderingComponents, camera_index: usize, shadow_system_bind_group_layout: &BindGroupLayout, shadow_system_bind_group: &BindGroup);
    fn clear(&mut self, game_state: &mut GameState);
}

pub struct RenderFeature3DHelper {
    camera_rendering: CameraRenderingComponents,
    features: Vec<Box<dyn RenderFeature3D>>,
}
impl RenderFeature3DHelper {
    pub fn new() -> RenderFeature3DHelper {
        RenderFeature3DHelper {
            camera_rendering: CameraRenderingComponents::new(1),
            features: vec![RenderFeatureDrawMesh::new()],
        }
    }
    pub fn set_graphics_mappings(&mut self, graphics_mappings: &[GraphicsMapping]) {
        self.camera_rendering = CameraRenderingComponents::new(graphics_mappings.len());
    }
    pub fn draw_3d_features(
        &mut self,
        graphics_mappings: &mut Vec<GraphicsMapping>,
        game_state: &mut Vec<GameState>,
        encoder: &mut egui_wgpu::wgpu::CommandEncoder,
        target_view: &mut egui_wgpu::wgpu::TextureView, // <-- changed from SurfaceTexture
        shadow_system: &ShadowSystem,
    ) {
        // generate a render pass for this instance
        let depth = SystemGPU::get_depth_texture();

        //
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
            depth_stencil_attachment: Self::get_depth_attatchment(&depth),
            timestamp_writes: None,
            occlusion_query_set: None, // keep or add depth if you use it
        });

        // iterate over each camera in state
        for i in 0..graphics_mappings.len() {
            //
            let game_state = game_state.get_mut(i).unwrap();
            let state_camera = game_state.get::<CameraState>();

            // get camera data
            let cur_camera_snapshot = &state_camera.cameras;
            let cur_graphics_mapping = &graphics_mappings[i];

            //
            if state_camera.resolution_height == 0 || state_camera.resolution_height == 0 {
                continue;
            }

            // create viewport bounds
            let viewport = Viewport::new(Vector2::new(state_camera.resolution_width as f32, state_camera.resolution_height as f32), cur_graphics_mapping.viewport_min, cur_graphics_mapping.viewport_max);

            // set the viewport based on mapping
            render_pass.set_viewport(viewport.x, viewport.y, viewport.width, viewport.height, 0.0, 1.0);

            // calculate camera binding values
            let camera_uniform = cur_camera_snapshot.get_uniform(viewport.width as i32, viewport.height as i32);

            //
            self.camera_rendering.update(i, &camera_uniform);
            let camera_rendering = &self.camera_rendering;

            // render features
            for feature in self.features.iter_mut() {
                feature.render(game_state, &mut render_pass, camera_rendering, i, &shadow_system.bind_group_layout, shadow_system.sampling_bind_group_for(i).unwrap());
            }
        }

        // cleanup
        for feature in self.features.iter_mut() {
            for game_state in &mut *game_state {
                feature.clear(game_state);
            }
        }
    }
    // get
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

pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub resolution_x: i32,
    pub resolution_y: i32,
}
impl Viewport {
    pub fn new(resolution: Vector2, min: Vector2, max: Vector2) -> Viewport {
        Viewport {
            resolution_x: f32::round(resolution.x) as i32,
            resolution_y: f32::round(resolution.y) as i32,
            x: min.x * resolution.x,
            y: min.y * resolution.y,
            width: ((max.x - min.x) * resolution.x).round(),
            height: ((max.y - min.y) * resolution.y).round(),
        }
    }
}
