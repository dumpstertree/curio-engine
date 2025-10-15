use super::super::collections::material::Material;
use super::asset::Asset;
use rusty_spine::{AnimationStateData, SkeletonData};
use std::sync::Arc;

// #[derive(Clone)]
pub struct ModelAssetAnimated {
    // pub skeleton: Skeleton,
    pub material: Arc<Material>,
    pub skeleton_data: Arc<SkeletonData>,
    pub state_data: Arc<AnimationStateData>,
}

// construction
impl ModelAssetAnimated {
    pub fn new(material: Arc<Material>, skeleton_data: Arc<SkeletonData>, state_data: Arc<AnimationStateData>) -> ModelAssetAnimated {
        ModelAssetAnimated { material, skeleton_data, state_data }
    }
}
// public
impl ModelAssetAnimated {}
// private
impl ModelAssetAnimated {}
// asset

impl Asset for ModelAssetAnimated {}
