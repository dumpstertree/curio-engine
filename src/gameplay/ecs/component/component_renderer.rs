use crate::IO::model_asset::Model_asset;

pub struct Renderer {
    asset: Option<Model_asset>,
}

impl Renderer {
    fn default() -> Renderer {
        Renderer { asset: None }
    }
}
