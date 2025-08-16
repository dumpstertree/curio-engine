use crate::egui_tools::EguiRenderer;
use core::collections::{event_queue::EventQueue, game_state::GameState};
use egui_wgpu::wgpu::{CommandEncoder, SurfaceTexture};

pub trait RenderFeature2D {
    fn render(
        &mut self,
        game_state: &mut GameState,
        system_event_queue: &mut EventQueue,
        output: &SurfaceTexture,
        encoder: &mut CommandEncoder,
        egui_renderer: &mut EguiRenderer,
    );
    fn clear(&mut self, game_state: &mut GameState);
}
