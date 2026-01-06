use core::{
    collections::color::Color,
    io::{asset_loader::AssetLoader, model_asset::ModelAsset, model_asset_animated::ModelAssetAnimated},
};
use std::sync::Arc;

use crate::{component::component_renderer_text::RendererCommon, field_override::FieldDeserialize, world_context::GameObject};

unsafe impl Send for RendererAnimated {}
unsafe impl Sync for RendererAnimated {}

pub struct RendererAnimated {
    cached_enabled_in_hierachy: bool,
    cached_tint_in_hierachy: Color,
    pub asset: Option<Arc<ModelAssetAnimated>>,
    pub mesh: Vec<Arc<ModelAsset>>,
    animation: String,
    parent: Option<GameObject>,
    enabled: bool,
    tint: Color,
    looping: bool,
    last_anim: String,
    last_frame_index: i32,
}
impl Default for RendererAnimated {
    fn default() -> Self {
        Self {
            cached_enabled_in_hierachy: Default::default(),
            cached_tint_in_hierachy: Default::default(),
            asset: Default::default(),
            mesh: Default::default(),
            animation: Default::default(),
            parent: Default::default(),
            enabled: true,
            tint: Color::white(),
            looping: Default::default(),
            last_anim: Default::default(),
            last_frame_index: Default::default(),
        }
    }
}

impl FieldDeserialize for RendererAnimated {
    fn override_field(&mut self, field: &str, value: &str) {
        match field {
            "asset" => self.asset = Some(AssetLoader::load_model_animated_from_database(value.to_string())),
            "enabled" => self.enabled = value.parse().unwrap_or_default(),
            "tint" => self.tint = value.parse().unwrap_or_default(),
            "animation" => self.animation = value.to_string(),
            "looping" => self.looping = value.parse().unwrap_or_default(),

            _ => {}
        }
    }
}
impl Clone for RendererAnimated {
    fn clone(&self) -> Self {
        Self {
            last_anim: String::new(),
            last_frame_index: -1,
            asset: self.asset.clone(),
            mesh: self.mesh.clone(),
            animation: self.animation.clone(),
            parent: self.parent.clone(),
            enabled: self.enabled.clone(),
            tint: self.tint.clone(),
            cached_enabled_in_hierachy: false,
            cached_tint_in_hierachy: Color::white(),
            looping: false,
        }
    }
}

impl RendererCommon for RendererAnimated {
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
}

impl RendererAnimated {
    pub fn default() -> RendererAnimated {
        RendererAnimated {
            last_anim: String::new(),
            last_frame_index: -1,
            asset: None,
            mesh: vec![],
            animation: String::new(),
            parent: None,
            enabled: true,
            tint: Color::white(),
            cached_enabled_in_hierachy: false,
            cached_tint_in_hierachy: Color::white(),
            looping: false,
        }
    }

    /// Set the current playing animation
    pub fn set_animation(&mut self, name: &str, looping: bool) -> &mut Self {
        self.looping = looping;
        self.animation = name.to_string();
        self
    }
    /// Set the asset
    pub fn set_asset(&mut self, asset: Option<Arc<ModelAssetAnimated>>) -> &mut Self {
        // set the asset
        self.asset = asset;

        // return this
        self
    }
    /// Updates the mesh to match the current animation state. This should only be called ONCE per frame.
    pub fn update_mesh(&mut self, time: f64) {
        // guard - no asset
        let Some(asset) = &self.asset else {
            return;
        };
        // get animation
        let anim_asset = asset.get_animation(&self.animation);

        // get the frame num for the time
        let frame_num = anim_asset.get_frame_num_for_normalized_time(time as f32, self.looping);

        // guard - is same animation and frame
        if self.last_anim == self.animation && frame_num as i32 == self.last_frame_index {
            return;
        }

        // get asset
        let frame_asset = anim_asset.frame_for_index(frame_num);

        //cache
        self.last_frame_index = frame_num as i32;
        self.last_anim = self.animation.clone();

        // assign anim
        self.mesh = frame_asset.mesh().to_vec();
    }
}
