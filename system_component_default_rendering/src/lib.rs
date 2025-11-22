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
use crate::render_feature_2d::RenderFeature2DHelper;
use crate::render_feature_3d::RenderFeature3DHelper;
use crate::render_feature_post_process::{PostProcessResources, RenderFeaturePostProcessHelper};
use crate::shadow_system::ShadowSystem;
use built_in_state::state_draw::DrawCallsState;
use built_in_state::state_sun::StateSun;
use core::collections::event_queue::EventQueue;
use core::collections::game_state::GameState;
use core::collections::matrix4x4::Matrix4x4;
use core::graphics::graphics_mapping::GraphicsMapping;
use core::system::system_component::SystemComponent;
use core::system::system_components::system_component_graphics::SystemComponentGraphics;
use core::system_adapters::adapter_system_gpu::SystemGPU;
use egui_wgpu::wgpu::{CommandEncoder, SurfaceTexture, TextureView};
use std::iter;
use winit::event::WindowEvent;

pub struct SystemComponentDefaultGraphics {
    shadow_system: ShadowSystem,
    offscreen_view: TextureView,
    render_feature_2d_helper: RenderFeature2DHelper,
    render_feature_3d_helper: RenderFeature3DHelper,
    render_feature_pp_helper: RenderFeaturePostProcessHelper,
    graphics_mappings: Vec<GraphicsMapping>,
    is_dirty: bool,
}

impl SystemComponent for SystemComponentDefaultGraphics {
    fn order(&self) -> i32 {
        9000
    }
    fn init(&mut self, _game_state: &mut Vec<GameState>) {}

    fn tick(&mut self, game_state: &mut Vec<GameState>, event_queue: &mut Vec<EventQueue>) {
        // initialize frame and get resources for rendering
        let drawing_resources = Self::initialize_frame();

        // seperate resourcees
        let mut encoder = drawing_resources.1;
        let output = drawing_resources.0;
        let output_view = drawing_resources.2;

        self.shadow_system
            .ensure_screens(game_state.len(), Matrix4x4::default());
        for i in 0..game_state.len() {
            let state_sun = game_state[i].get::<StateSun>();
            if state_sun.cast_shadows {
                self.shadow_system
                    .update_for_screen(i, &state_sun.direction);
                self.shadow_system
                    .render_for_screen(&mut encoder, i, &game_state[i].get::<DrawCallsState>().draw_calls);
            }
        }

        // draw 3D into offscreen
        self.render_feature_3d_helper
            .draw_3d_features(&mut self.graphics_mappings, game_state, &mut encoder, &mut self.offscreen_view, &self.shadow_system);

        // post-processing into swapchain output
        self.render_feature_pp_helper
            .draw_post_features(&mut encoder, &self.offscreen_view, &output_view);

        // draw all 2d
        self.render_feature_2d_helper
            .draw_2d_features(game_state, &mut self.graphics_mappings, &mut encoder, &output, event_queue);

        // finalize the frame and render to the gpu
        Self::finalize_frame(output, encoder);
    }

    fn raw_event(&mut self, event: WindowEvent) {
        let window = SystemGPU::get_window();
        // self.egui_renderer.handle_input(&window, &event);
    }
    fn set_game_mode(&mut self, _: &mut Vec<GameState>, game_mode: &core::dumpster_engine::GameMode) {
        let mut graphics_mapping = vec![];
        for x in &game_mode.game_instances {
            graphics_mapping.push(x.graphics_mappings.clone());
        }

        self.render_feature_3d_helper
            .set_graphics_mappings(&graphics_mapping);

        self.graphics_mappings = graphics_mapping;
        self.is_dirty = true;

        println!("setting game mode, {}", game_mode.game_instances.len());
    }
}
impl SystemComponentGraphics for SystemComponentDefaultGraphics {}
impl SystemComponentDefaultGraphics {
    // static
    pub fn register_feature_driver() {}

    // construction
    pub fn new() -> Box<SystemComponentDefaultGraphics> {
        // generate a texture to pass between RenderDrivers that can be written to
        let rt = Self::generate_render_texture();

        Box::new(SystemComponentDefaultGraphics {
            is_dirty: true,
            graphics_mappings: Vec::new(),
            shadow_system: ShadowSystem::new(Matrix4x4::default(), 1),
            render_feature_2d_helper: RenderFeature2DHelper::new(),
            render_feature_3d_helper: RenderFeature3DHelper::new(),
            render_feature_pp_helper: RenderFeaturePostProcessHelper::new(&rt.1),
            offscreen_view: rt.1,
        })
    }

    // frame lifecycle
    pub fn initialize_frame() -> (SurfaceTexture, CommandEncoder, TextureView) {
        // get Surface and Device from the GPU
        let surface = &SystemGPU::get_surface();
        let device = &SystemGPU::get_device();

        // create the output
        let output = surface.get_current_texture().unwrap();
        let encoder = device.create_command_encoder(&egui_wgpu::wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });

        // draw all post-process
        let output_view = output
            .texture
            .create_view(&egui_wgpu::wgpu::TextureViewDescriptor::default());

        (output, encoder, output_view)
    }
    pub fn finalize_frame(output: SurfaceTexture, encoder: CommandEncoder) {
        // get queue from the GPU
        let queue = SystemGPU::get_queue();

        // submit commands for execution
        queue.submit(iter::once(encoder.finish()));

        // present the completed texture
        output.present();
    }

    // dependency
    pub fn generate_render_texture() -> (egui_wgpu::wgpu::Texture, TextureView) {
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

        (offscreen_texture, offscreen_view)
    }
}

pub trait RenderFeatureDriver {
    fn new(render_texture: &TextureView);
    fn raw_event(_event: WindowEvent) {}
    fn set_graphics_mappings(_graphic_mappings: &[GraphicsMapping]) {}
}
