use curio_core::{Assets, Color, FieldState};
use facet::facet;
use gameplay::traits::field_override::FieldOverride;
use rendering_static::ModelAsset;
use std::sync::Arc;

use crate::asset::model_asset_animated::ModelAssetAnimated;

#[facet]
pub struct RendererDynamic {
    pub asset: Option<Arc<ModelAssetAnimated>>,
    pub mesh: Vec<Arc<ModelAsset>>,
    animation: String,
    pub enabled: bool,
    pub tint: Color,
    looping: bool,
    last_anim: String,
    last_frame_index: i32,
}
impl RendererDynamic {
    pub fn default() -> RendererDynamic {
        Self {
            asset: Default::default(),
            mesh: Default::default(),
            animation: Default::default(),
            enabled: true,
            tint: Color::white(),
            looping: Default::default(),
            last_anim: Default::default(),
            last_frame_index: Default::default(),
            owner: None,
        }
    }
    pub fn builder() -> RendererDynamicBuilder {
        RendererDynamicBuilder {
            asset: None,
            animation: String::new(),
            looping: false,
            enabled: true,
            tint: Color::white(),
        }
    }
}
impl FieldOverride for RendererDynamic {
    fn apply(&mut self, field: &str, value: &str) {
        let value = value.trim();
        match field {
            "asset" => {
                // ignore unknown names rather than panicking on the lookup
                if let Some(uid) = Assets::try_lookup_key_for_name(value) {
                    self.asset = Some(Assets::load_asset::<ModelAssetAnimated>(&uid));
                }
            }
            "enabled" => self.enabled = value.parse().unwrap_or_default(),
            "tint" => self.tint = value.parse().unwrap_or(self.tint),
            "animation" => self.animation = value.to_string(),
            "looping" => self.looping = value.parse().unwrap_or_default(),
            _ => {}
        }
    }
    fn get_state(&self) -> Vec<FieldState> {
        // NOTE: asset is intentionally not serialized. Compositions reference the asset by name
        // ("asset: <name>"), but no name is stored on the loaded asset, so a placeholder here would
        // be fed back through apply() and fail the lookup (see FormBuilder3D::facet re-applying
        // state onto an existing facet).
        vec![
            FieldState::new("enabled", self.enabled),
            FieldState::new("tint", self.tint),
            FieldState::new("animation", self.animation.clone()),
            FieldState::new("looping", self.looping),
        ]
    }
}
impl RendererDynamic {
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

pub struct RendererDynamicBuilder {
    asset: Option<Arc<ModelAssetAnimated>>,
    animation: String,
    looping: bool,
    enabled: bool,
    tint: Color,
}
impl RendererDynamicBuilder {
    pub fn asset(mut self, asset: Option<Arc<ModelAssetAnimated>>) -> Self {
        self.asset = asset;
        self
    }
    pub fn animation(mut self, name: &str, looping: bool) -> Self {
        self.animation = name.to_string();
        self.looping = looping;
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn tint(mut self, tint: Color) -> Self {
        self.tint = tint;
        self
    }
    pub fn build(self) -> RendererDynamic {
        RendererDynamic {
            asset: self.asset,
            mesh: Vec::new(),
            animation: self.animation,
            enabled: self.enabled,
            tint: self.tint,
            looping: self.looping,
            last_anim: String::new(),
            last_frame_index: -1,
            owner: None,
        }
    }
}
