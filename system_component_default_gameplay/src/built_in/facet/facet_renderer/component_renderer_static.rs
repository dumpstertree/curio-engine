use crate::{built_in::facet::facet_renderer::component_renderer_text::RendererCommon, gameobject::GameObject, traits::field_override::FieldOverride};
use core::{
    collections::color::Color,
    io::{asset_loader::AssetLoader, model_asset::ModelAsset},
};
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct Renderer {
    pub asset: Option<Arc<ModelAsset>>,
    parent: Option<GameObject>,
    enabled: bool,
    tint: Color,
    cached_enabled_in_hierachy: bool,
    cached_tint_in_hierachy: Color,
}

impl FieldOverride for Renderer {
    fn apply(&mut self, field: &str, value: &str) {
        match field {
            "asset" => self.asset = Some(AssetLoader::load_model_static_from_database(value.to_string())),
            "enabled" => self.enabled = value.parse().unwrap_or_default(),
            "tint" => self.tint = value.parse().unwrap_or_default(),
            _ => {}
        }
    }
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
            cached_enabled_in_hierachy: true,
            cached_tint_in_hierachy: Color::white(),
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

    fn set_cached_enabled_in_hierarchy(&mut self, val: bool) {
        self.cached_enabled_in_hierachy = val;
    }

    fn get_cached_enabled_in_hierarchy(&self) -> bool {
        self.cached_enabled_in_hierachy
    }

    fn set_cached_tint_in_hierarchy(&mut self, val: Color) {
        self.cached_tint_in_hierachy = val;
    }

    fn get_cached_tint_in_hierarchy(&self) -> Color {
        self.cached_tint_in_hierachy
    }
}
