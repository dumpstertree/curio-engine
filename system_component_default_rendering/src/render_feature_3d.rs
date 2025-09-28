use core::collections::game_state::GameState;
use egui_wgpu::wgpu::RenderPass;

use crate::{camera_rendering_components::CameraRenderingComponents, shadow_system::ShadowSystem};

pub trait RenderFeature3D {
    fn render(&mut self, game_state: &mut GameState, render_pass: &mut RenderPass, camera: &CameraRenderingComponents, camera_index: usize, shadow_system: &ShadowSystem);
    fn clear(&mut self, game_state: &mut GameState);
}
