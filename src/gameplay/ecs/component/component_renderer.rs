use std::sync::Arc;

use crate::IO::model_asset::Model_asset;

#[derive(Clone)]
pub struct Renderer {
    pub asset: Option<Arc<Model_asset>>,
}

impl Renderer {
    pub fn default() -> Renderer {
        Renderer { asset: None }
    }
    pub fn set_asset(mut self, asset: Option<Arc<Model_asset>>) -> Renderer {
        self.asset = asset;
        self
    }
}
