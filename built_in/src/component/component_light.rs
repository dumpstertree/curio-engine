use core::{
    collections::light_uniform::{DrawCallLight, LightType},
    io::model_asset::ModelAsset,
};
use std::sync::Arc;

pub struct ComponentLight {
    pub asset: LightType,
}

impl ComponentLight {
    pub fn default() -> ComponentLight {
        ComponentLight { asset: LightType::Directional }
    }
}
