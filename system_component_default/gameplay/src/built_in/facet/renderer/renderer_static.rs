use crate::{
    built_in::facet::renderer_common::RendererCommon,
    form::Form,
    traits::{facet_common::FacetCommon, field_override::FieldOverride},
};
use curio_core::{
    collections::color::Color,
    io::{asset_loader::AssetLoader, model_asset::ModelAsset},
};
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct RendererStatic {
    pub asset: Option<Arc<ModelAsset>>,
    enabled: bool,
    tint: Color,
    cached_enabled_in_hierachy: bool,
    cached_tint_in_hierachy: Color,
    owner: Option<Form>,
}

impl FieldOverride for RendererStatic {
    fn apply(&mut self, field: &str, value: &str) {
        match field {
            "asset" => self.asset = Some(AssetLoader::load_model_static_from_database(&AssetLoader::try_lookup_key_for_name(value).unwrap())),
            "enabled" => self.enabled = value.parse().unwrap_or_default(),
            "tint" => self.tint = value.parse().unwrap_or_default(),
            _ => {}
        }
    }
}

unsafe impl Send for RendererStatic {}
unsafe impl Sync for RendererStatic {}

impl RendererStatic {
    pub fn default() -> RendererStatic {
        RendererStatic {
            asset: None,
            enabled: true,
            tint: Color::white(),
            cached_enabled_in_hierachy: true,
            cached_tint_in_hierachy: Color::white(),
            owner: None,
        }
    }
    pub fn set_asset(mut self, asset: Option<Arc<ModelAsset>>) -> Self {
        self.asset = Self::get_model_asset(asset, self.tint);
        self
    }
}
impl FacetCommon for RendererStatic {
    fn set_ownership(&mut self, owner: Form) {
        self.owner = Some(owner);
    }
    fn form(&self) -> Form {
        self.owner.clone().unwrap()
    }
}
impl RendererCommon for RendererStatic {
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn get_enabled(&self) -> bool {
        self.enabled
    }

    fn set_tint(&mut self, tint: Color) {
        self.asset = Self::get_model_asset(self.asset.clone(), tint);
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
impl RendererStatic {
    fn get_model_asset(asset: Option<Arc<ModelAsset>>, tint: Color) -> Option<Arc<ModelAsset>> {
        // no asset
        let Some(asset) = asset else {
            return None;
        };

        // no tint
        if tint == Color::white() {
            return Some(asset);
        }

        // edit material to include tint
        let mut mats = Vec::new();
        for mat in &asset.materials {
            let mut m = mat.instantiate("new");
            m.set_color_with_label(tint, "tint");
            m.finalize();
            mats.push(Arc::new(m));
        }

        // return edited
        Some(Arc::new(ModelAsset::new(asset.mesh.clone(), mats)))
    }
}
