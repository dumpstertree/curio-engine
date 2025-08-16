use core::collections::game_state::GameState;
use egui_wgpu::wgpu::{BindGroup, BindGroupLayout, RenderPass};

pub trait RenderFeature3D {
    fn render(
        &mut self,
        game_state: &mut GameState,
        render_pass: &mut RenderPass,
        camera_bind_group: &BindGroup,
        camera_bind_group_layout: &BindGroupLayout,
    );
    fn clear(&mut self, game_state: &mut GameState);
}
