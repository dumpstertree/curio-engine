use crate::camera_rendering_components::CameraRenderingComponents;
use core::collections::game_state::GameState;
use egui_wgpu::wgpu::RenderPass;

pub trait RenderFeaturePostProcess {
    fn render(
        &mut self,
        game_state: &mut GameState,
        render_pass: &mut egui_wgpu::wgpu::RenderPass<'_>,
        input_view: &egui_wgpu::wgpu::TextureView, // ← offscreen texture from 3D pass
    );

    fn clear(&mut self, game_state: &mut GameState);
}
