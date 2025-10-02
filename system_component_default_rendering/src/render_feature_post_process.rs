use core::{collections::game_state::GameState, system_adapters::adapter_system_gpu::SystemGPU};
use std::sync::Arc;

use egui_wgpu::{Texture, wgpu::TextureView};

use crate::render_feature_post_processes::{render_feature_post_process_fog::RenderFeaturePostProcessFog, render_feature_post_process_kuwahara::RenderFeaturePostProcessKuwahara, render_feature_post_process_sobel_outline::RenderFeaturePostProcessOutline};

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
pub struct RenderFeaturePostProcessHelper {
    pp_resource: PostProcessResources,
    features: Vec<Box<dyn RenderFeaturePostProcess>>,
}
impl RenderFeaturePostProcessHelper {
    pub fn new(offscreen_view: &TextureView) -> RenderFeaturePostProcessHelper {
        // get resources from GPU
        let config = SystemGPU::get_config();
        let device = SystemGPU::get_device();
        let depth_view = &SystemGPU::get_depth_texture().view;

        let pp_resource = PostProcessResources::new(device.clone(), config.width, config.height, config.format);
        let features: Vec<Box<dyn RenderFeaturePostProcess>> = vec![
            RenderFeaturePostProcessKuwahara::new(device.clone(), config.format, &pp_resource, depth_view, &offscreen_view),
            RenderFeaturePostProcessOutline::new(device.clone(), config.format, &pp_resource, depth_view, &offscreen_view),
            RenderFeaturePostProcessFog::new(device.clone(), config.format, &pp_resource, depth_view, &offscreen_view),
        ];
        // construct -> return
        RenderFeaturePostProcessHelper { pp_resource, features }
    }
    pub fn draw_post_features(
        &mut self,
        encoder: &mut egui_wgpu::wgpu::CommandEncoder,
        input_view: &egui_wgpu::wgpu::TextureView,  // offscreen scene render
        output_view: &egui_wgpu::wgpu::TextureView, // final swapchain target
    ) {
        use crate::render_feature_post_process::PostProcessSource;

        // src starts as the offscreen scene
        let mut src: &egui_wgpu::wgpu::TextureView = input_view;

        // ping/pong targets — keep them distinct and swap between them
        let mut ping: &egui_wgpu::wgpu::TextureView = &self.pp_resource.view_a;
        let mut pong: &egui_wgpu::wgpu::TextureView = &self.pp_resource.view_b;

        // track logical source for bind-group selection
        let mut current_source = PostProcessSource::Offscreen;

        let post_features_len = self.features.len();

        for (i, feature) in self.features.iter_mut().enumerate() {
            let is_last = i == post_features_len - 1;

            // destination: final swapchain if last pass, otherwise ping
            let target: &egui_wgpu::wgpu::TextureView = if is_last { output_view } else { ping };

            // run the pass
            feature.render(encoder, src, target, current_source);

            if !is_last {
                // advance the pipeline:
                // next src is what we just wrote (target)
                src = target;

                // advance logical source for bind-group selection
                current_source = match current_source {
                    PostProcessSource::Offscreen => PostProcessSource::ViewA,
                    PostProcessSource::ViewA => PostProcessSource::ViewB,
                    PostProcessSource::ViewB => PostProcessSource::ViewA,
                };

                // swap ping/pong so next write goes into the other ping-pong texture
                std::mem::swap(&mut ping, &mut pong);
            }
        }
    }
}

pub struct PostProcessResources {
    pub texture_a: egui_wgpu::wgpu::Texture,
    pub view_a: egui_wgpu::wgpu::TextureView,
    pub texture_b: egui_wgpu::wgpu::Texture,
    pub view_b: egui_wgpu::wgpu::TextureView,
}

impl PostProcessResources {
    pub fn new(device: Arc<egui_wgpu::wgpu::Device>, width: u32, height: u32, format: egui_wgpu::wgpu::TextureFormat) -> Self {
        let usage = egui_wgpu::wgpu::TextureUsages::RENDER_ATTACHMENT | egui_wgpu::wgpu::TextureUsages::TEXTURE_BINDING;
        let desc = |label| egui_wgpu::wgpu::TextureDescriptor {
            label: Some(label),
            size: egui_wgpu::wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: egui_wgpu::wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        };

        let texture_a = device.create_texture(&desc("post A"));
        let view_a = texture_a.create_view(&egui_wgpu::wgpu::TextureViewDescriptor::default());

        let texture_b = device.create_texture(&desc("post B"));
        let view_b = texture_b.create_view(&egui_wgpu::wgpu::TextureViewDescriptor::default());

        Self { texture_a, view_a, texture_b, view_b }
    }
}
