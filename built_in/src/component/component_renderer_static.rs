use core::io::model_asset::ModelAsset;
use std::sync::Arc;

// #[derive(Clone)]
pub struct Renderer {
    pub asset: Option<Arc<ModelAsset>>,
}

impl Renderer {
    pub fn default() -> Renderer {
        Renderer { asset: None }
    }
    pub fn set_asset(mut self, asset: Option<Arc<ModelAsset>>) -> Self {
        self.asset = asset;
        self
    }
}
