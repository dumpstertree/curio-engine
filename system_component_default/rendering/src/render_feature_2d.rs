use crate::{egui_tools::EguiRenderer, render_feature_2ds::render_feature_draw_ui::RenderFeatureDrawUI};
use curio_core::{
    GraphicsMapping,
    collections::{event_queue::EventQueue, ledger::Ledger},
    system_adapters::adapter_system_gpu::SystemGPU,
};
use egui_wgpu::wgpu::{CommandEncoder, SurfaceTexture};

pub trait RenderFeature2D {
    fn render(&mut self, ledger: &mut Ledger, system_event_queue: &mut EventQueue, output: &SurfaceTexture, encoder: &mut CommandEncoder, egui_renderer: &mut EguiRenderer);
    fn clear(&mut self, ledger: &mut Ledger);
}

pub struct RenderFeature2DHelper {
    egui_renderer: EguiRenderer,
    features: Vec<Box<dyn RenderFeature2D>>,
}
impl RenderFeature2DHelper {
    pub fn new() -> RenderFeature2DHelper {
        let c = SystemGPU::get_config();
        let w = &(*SystemGPU::get_window());
        let d = &(*SystemGPU::get_device());
        RenderFeature2DHelper {
            egui_renderer: EguiRenderer::new(d, c.format, None, 1, w),
            features: vec![RenderFeatureDrawUI::new()],
        }
    }
    pub fn draw_2d_features(&mut self, ledger: &mut Vec<Ledger>, graphics_mappings: &mut Vec<GraphicsMapping>, encoder: &mut CommandEncoder, output: &SurfaceTexture, event_queue: &mut Vec<EventQueue>) {
        // THIS IS HACKED BECAUSE WE CANT ALL WRITE TO THE MAIN SCREEN

        // for i in 0..(self.graphics_mappings.len() as usize) {
        let i = graphics_mappings.len() - 1;
        let ledger = ledger.get_mut(i).unwrap();
        let event_queue = event_queue.get_mut(i).unwrap();
        for feature in self.features.iter_mut() {
            feature.render(ledger, event_queue, &output, encoder, &mut self.egui_renderer);
        }

        for feature in self.features.iter_mut() {
            feature.clear(ledger);
        }
        // }
    }
}
