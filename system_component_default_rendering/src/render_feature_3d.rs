use core::collections::game_state::GameState;
use egui_wgpu::wgpu::{BindGroup, BindGroupLayout, RenderPass};

use crate::camera_rendering_components::CameraRenderingComponents;

pub trait RenderFeature3D {
    fn render(&mut self, game_state: &mut GameState, render_pass: &mut RenderPass, camera: &CameraRenderingComponents, camera_index: usize);
    fn clear(&mut self, game_state: &mut GameState);
}
