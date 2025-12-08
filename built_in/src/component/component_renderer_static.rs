use core::{collections::color::Color, gameplay::world_context::GameObject, io::model_asset::ModelAsset};
use std::sync::Arc;

use hecs::Entity;

use crate::component::component_renderer_text::RendererCommon;

// #[derive(Clone)]
pub struct Renderer {
    pub asset: Option<Arc<ModelAsset>>,
    parent: Option<GameObject>,
    enabled: bool,
    tint: Color,
}

unsafe impl Send for Renderer {}
unsafe impl Sync for Renderer {}

impl Renderer {
    pub fn default() -> Renderer {
        Renderer {
            asset: None,
            parent: None,
            enabled: true,
            tint: Color::white(),
        }
    }
    pub fn set_asset(mut self, asset: Option<Arc<ModelAsset>>) -> Self {
        self.asset = asset;
        self
    }
}
impl RendererCommon for Renderer {
    fn set_parent(&mut self, parent: Option<GameObject>) {
        self.parent = parent;
    }

    fn get_parent(&self) -> Option<GameObject> {
        self.parent.clone()
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn get_enabled(&self) -> bool {
        self.enabled
    }

    fn set_tint(&mut self, tint: Color) {
        self.tint = tint;
    }

    fn get_tint(&self) -> Color {
        self.tint
    }
}
