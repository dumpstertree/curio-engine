// One painted form: instead of a Renderer pushing a DrawCall, a PaintOil
// facet says "draw me as brush dabs". The mesh-sampling habit reads these,
// and the dab pass draws the result — painted forms contribute NO DrawCall
// and (so far) cast no shadow; if dabs should ever cast, the place to wire
// it is a cast_shadow DrawCall pushed by the habit into SysRecordRendering
// while suppressing its color draw.
//
// asset is loaded by name through the same lookup RendererStatic uses;
// unknown names are ignored rather than panicking.

use curio_core::{Assets, Color, FieldState};
use rendering_static::ModelAsset;
use std::sync::Arc;

use facet::facet;
use gameplay::traits::field_override::FieldOverride;

#[facet]
pub struct PaintOil {
    pub asset: Option<Arc<ModelAsset>>,
    pub enabled: bool,
    /// target stroke length in metres — density follows surface area
    pub dab_size: f32,
    /// saturated tints bias the dab palette toward the tint's hue;
    /// white/gray lets every dab draw its own slot
    pub tint: Color,
}

impl PaintOil {
    pub fn default() -> PaintOil {
        PaintOil {
            asset: None,
            enabled: true,
            dab_size: 0.18,
            tint: Color::white(),
            owner: None,
        }
    }
    pub fn builder() -> PaintOilBuilder {
        PaintOilBuilder {
            asset: None,
            enabled: true,
            dab_size: 0.18,
            tint: Color::white(),
        }
    }
}

impl FieldOverride for PaintOil {
    fn apply(&mut self, field: &str, value: &str) {
        let value = value.trim();
        match field {
            "asset" => {
                // ignore unknown names rather than panicking on the lookup (matches RendererStatic/RendererDynamic)
                if let Some(uid) = Assets::try_lookup_key_for_name(value) {
                    self.asset = Some(Assets::load_asset::<ModelAsset>(&uid));
                }
            }
            "enabled" => self.enabled = value.parse().unwrap_or_default(),
            "dab_size" => self.dab_size = value.parse().unwrap_or(self.dab_size),
            "tint" => self.tint = value.parse().unwrap_or(self.tint),
            _ => {}
        }
    }
    fn get_state(&self) -> Vec<FieldState> {
        // NOTE: asset is intentionally not serialized here — same round-trip
        // hazard RendererStatic documents (no uid to re-apply through apply())
        vec![
            FieldState::new("enabled", self.enabled),
            FieldState::new("dab_size", self.dab_size),
            FieldState::new("tint", self.tint),
        ]
    }
}

pub struct PaintOilBuilder {
    asset: Option<Arc<ModelAsset>>,
    enabled: bool,
    dab_size: f32,
    tint: Color,
}

impl PaintOilBuilder {
    pub fn asset(mut self, asset: Option<Arc<ModelAsset>>) -> Self {
        self.asset = asset;
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn dab_size(mut self, dab_size: f32) -> Self {
        self.dab_size = dab_size;
        self
    }
    pub fn tint(mut self, tint: Color) -> Self {
        self.tint = tint;
        self
    }
    pub fn build(self) -> PaintOil {
        PaintOil {
            asset: self.asset,
            enabled: self.enabled,
            dab_size: self.dab_size,
            tint: self.tint,
            owner: None,
        }
    }
}
