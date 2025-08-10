use std::sync::Arc;

use core::io::model_asset::ModelAsset;

#[derive(Clone)]
pub struct Renderer {
    pub asset: Option<Arc<ModelAsset>>,
}

impl Renderer {
    pub fn default() -> Renderer {
        Renderer { asset: None }
    }
    pub fn set_asset(mut self, asset: Option<Arc<ModelAsset>>) -> Renderer {
        self.asset = asset;
        self
    }
}
