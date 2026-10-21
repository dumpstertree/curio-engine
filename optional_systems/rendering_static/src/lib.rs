pub fn main() {}

pub use crate::asset::model_asset::ModelAsset;
pub use crate::facet::renderer_static::RendererStatic;
pub use crate::primitives::{PrimitiveResolution, PrimitiveType, Primitives};

pub mod habit {
    pub(crate) mod habit_update;
    pub(crate) mod system_renderer_update_state;
}
pub mod facet {
    pub(crate) mod renderer_static;
}

pub mod asset {
    pub(crate) mod model_asset;
}

pub mod primitives;
