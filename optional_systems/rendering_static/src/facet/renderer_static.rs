use crate::ModelAsset;
use curio_core::{Color, FieldState, io::asset_loader::Assets};
use facet::facet;
use gameplay::traits::field_override::FieldOverride;
use std::sync::Arc;

#[facet]
pub struct RendererStatic {
    pub asset: Option<Arc<ModelAsset>>,
    pub tint: Color,
}

impl FieldOverride for RendererStatic {
    fn apply(&mut self, field: &str, value: &str) {
        println!("value :{}", value);
        let value = value.trim();
        match field {
            "asset" => self.asset = Some(Assets::load_asset::<ModelAsset>(&value.parse().unwrap_or_default())),
            "tint" => self.tint = value.parse().unwrap_or_default(),
            _ => {}
        }
    }
    fn get_state(&self) -> Vec<FieldState> {
        vec![
            FieldState::new("asset", "TODO"), //
            FieldState::new("tint", self.tint),
        ]
    }
}

impl RendererStatic {
    pub fn default() -> RendererStatic {
        RendererStatic { asset: None, tint: Color::white(), owner: None }
    }
    pub fn builder() -> RendererStaticBuilder {
        RendererStaticBuilder { asset: None, tint: Color::white() }
    }
    pub fn set_asset(mut self, asset: Option<Arc<ModelAsset>>) -> Self {
        self.asset = Self::get_model_asset(asset, self.tint);
        self
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

pub struct RendererStaticBuilder {
    asset: Option<Arc<ModelAsset>>,
    tint: Color,
}

impl RendererStaticBuilder {
    pub fn asset(mut self, asset: Option<Arc<ModelAsset>>) -> Self {
        self.asset = asset;
        self
    }
    pub fn tint(mut self, tint: Color) -> Self {
        self.tint = tint;
        self
    }
    pub fn build(self) -> RendererStatic {
        RendererStatic { asset: self.asset, tint: self.tint, owner: None }
    }
}
