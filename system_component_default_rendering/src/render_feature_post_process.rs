use core::collections::game_state::GameState;

pub trait RenderFeaturePostProcess {
    fn render(&mut self, encoder: &mut egui_wgpu::wgpu::CommandEncoder, input_view: &egui_wgpu::wgpu::TextureView, output_view: &egui_wgpu::wgpu::TextureView, source: PostProcessSource);

    fn clear(&mut self, game_state: &mut GameState);
}

#[derive(Clone, Copy, Debug)]
pub enum PostProcessSource {
    Offscreen, // original 3D render target
    ViewA,     // ping-pong view A
    ViewB,     // ping-pong view B
}
