use egui_wgpu::wgpu::{Adapter, Device, Queue, Surface, SurfaceConfiguration};
use std::sync::Arc;
use winit::window::Window;

use crate::TextureAsset;

pub struct GPUInstance {
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub surface: Arc<Surface<'static>>,
    pub adapter: Arc<Adapter>,
    pub window: Arc<Window>,
    pub config: Arc<SurfaceConfiguration>,
    pub depth_texture: Arc<TextureAsset>,
}
