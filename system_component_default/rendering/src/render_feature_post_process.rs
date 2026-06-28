use curio_core::{Ledger, TextureAsset, services};
use std::sync::Arc;

use egui_wgpu::wgpu::{AddressMode, CompareFunction, Device, Extent3d, FilterMode, SamplerDescriptor, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor};

use crate::render_feature_post_processes::{render_feature_post_process_fog::RenderFeaturePostProcessFog, render_feature_post_process_kuwahara::RenderFeaturePostProcessKuwahara, render_feature_post_process_sobel_outline::RenderFeaturePostProcessOutline};

pub trait RenderFeaturePostProcess {
    fn render(&mut self, encoder: &mut egui_wgpu::wgpu::CommandEncoder, input_view: &TextureView, output_view: &TextureView, source: PostProcessSource);
    fn clear(&mut self, ledger: &mut Ledger);
}

#[derive(Clone, Copy, Debug)]
pub enum PostProcessSource {
    Offscreen,
    ViewA,
    ViewB,
}

pub struct RenderFeaturePostProcessHelper {
    pp_resource: PostProcessResources,
    features: Vec<Box<dyn RenderFeaturePostProcess>>,
    // Owned depth texture — written by 3D pass, read by post-process features
    pub depth_texture: TextureAsset,
}

impl RenderFeaturePostProcessHelper {
    pub fn new(offscreen_view: &TextureView) -> RenderFeaturePostProcessHelper {
        let s = services();
        let device = Arc::new(s.gpu.device().clone());

        let width = s.gpu.capture_width;
        let height = s.gpu.capture_height;
        let format = TextureFormat::Rgba8UnormSrgb; // TextureFormat added to GpuHandle

        // Generate depth texture locally — no longer pulled from services
        let depth_texture = Self::create_depth_texture(&device, width, height);
        let depth_view = &depth_texture.view;

        let pp_resource = PostProcessResources::new(device.clone(), width, height, format);

        let features: Vec<Box<dyn RenderFeaturePostProcess>> = vec![
            RenderFeaturePostProcessKuwahara::new(device.clone(), format, &pp_resource, depth_view, offscreen_view),
            RenderFeaturePostProcessOutline::new(device.clone(), format, &pp_resource, depth_view, offscreen_view),
            RenderFeaturePostProcessFog::new(device.clone(), format, &pp_resource, depth_view, offscreen_view),
        ];

        RenderFeaturePostProcessHelper { pp_resource, features, depth_texture }
    }

    fn create_depth_texture(device: &Device, width: u32, height: u32) -> TextureAsset {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("depth_texture"),
            size: Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureAsset::DEPTH_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&TextureViewDescriptor::default());

        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            compare: Some(CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        TextureAsset { texture, view, sampler }
    }

    pub fn draw_post_features(&mut self, encoder: &mut egui_wgpu::wgpu::CommandEncoder, input_view: &TextureView, output_view: &TextureView) {
        let mut src = input_view;
        let mut ping = &self.pp_resource.view_a;
        let mut pong = &self.pp_resource.view_b;
        let mut current_source = PostProcessSource::Offscreen;
        let post_features_len = self.features.len();

        for (i, feature) in self.features.iter_mut().enumerate() {
            let is_last = i == post_features_len - 1;
            let target = if is_last { output_view } else { ping };

            feature.render(encoder, src, target, current_source);

            if !is_last {
                src = target;
                current_source = match current_source {
                    PostProcessSource::Offscreen => PostProcessSource::ViewA,
                    PostProcessSource::ViewA => PostProcessSource::ViewB,
                    PostProcessSource::ViewB => PostProcessSource::ViewA,
                };
                std::mem::swap(&mut ping, &mut pong);
            }
        }
    }
}

pub struct PostProcessResources {
    pub texture_a: egui_wgpu::wgpu::Texture,
    pub view_a: TextureView,
    pub texture_b: egui_wgpu::wgpu::Texture,
    pub view_b: TextureView,
}

impl PostProcessResources {
    pub fn new(device: Arc<Device>, width: u32, height: u32, format: TextureFormat) -> Self {
        let usage = TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING;
        let desc = |label| TextureDescriptor {
            label: Some(label),
            size: Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        };

        let texture_a = device.create_texture(&desc("post A"));
        let view_a = texture_a.create_view(&TextureViewDescriptor::default());
        let texture_b = device.create_texture(&desc("post B"));
        let view_b = texture_b.create_view(&TextureViewDescriptor::default());

        Self { texture_a, view_a, texture_b, view_b }
    }
}
