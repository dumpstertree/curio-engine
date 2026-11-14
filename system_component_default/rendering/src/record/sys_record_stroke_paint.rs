use curio_core::{FieldState, Ledger, RecordOverride, RecordScope};
use record::record;

// tuning for the stamp layer (the kuwahara guide is tuned by
// SysRecordPostProcess); sizes are in pixels of the full capture target
#[record(name = "StrokePaint", ownership = RecordScope::Instance)]
pub struct SysRecordStrokePaint {
    pub size: f32,
    pub size_min: f32,
    pub mag_scale: f32,
    pub elongation: f32,
    pub aspect_min: f32,
    pub alpha: f32,
    pub feather: f32,
    pub keep: f32,
    pub jitter: f32,
    pub wobble: f32,
    pub seed: f32,
}

impl SysRecordStrokePaint {
    // defaults mirror RenderFeaturePostProcessPainterly::new()'s constructor
    pub fn default() -> SysRecordStrokePaint {
        SysRecordStrokePaint {
            size: 22.0, // long-axis daub in flat areas
            size_min: 0.35, // busiest areas keep ~a third of that length
            mag_scale: 10.0, // contrast sensitivity against the smoothed guide
            elongation: 2.2, // flat areas: long daubs held along edges
            aspect_min: 1.4, // busy areas: rounder small daubs
            alpha: 1.0,
            feather: 0.45, // soft edge fraction of the analytic brush
            keep: 1.0, // every slot draws; lower it thins the paint
            jitter: 0.45,
            wobble: 0.25,
            seed: 0.0,
        }
    }
}

impl RecordOverride for SysRecordStrokePaint {
    fn set_state(&mut self, field: &str, val: &str) {
        let Some(val) = val.parse::<f32>().ok() else { return };
        match field {
            "size" => { self.size = val }
            "size_min" => { self.size_min = val }
            "mag_scale" => { self.mag_scale = val }
            "elongation" => { self.elongation = val }
            "aspect_min" => { self.aspect_min = val }
            "alpha" => { self.alpha = val }
            "feather" => { self.feather = val }
            "keep" => { self.keep = val }
            "jitter" => { self.jitter = val }
            "wobble" => { self.wobble = val }
            "seed" => { self.seed = val }
            _ => {}
        }
    }
    fn get_state(&self) -> Vec<FieldState> {
        vec![
            FieldState::new("size", self.size),
            FieldState::new("size_min", self.size_min),
            FieldState::new("mag_scale", self.mag_scale),
            FieldState::new("elongation", self.elongation),
            FieldState::new("aspect_min", self.aspect_min),
            FieldState::new("alpha", self.alpha),
            FieldState::new("feather", self.feather),
            FieldState::new("keep", self.keep),
            FieldState::new("jitter", self.jitter),
            FieldState::new("wobble", self.wobble),
            FieldState::new("seed", self.seed),
        ]
    }
}

pub trait ExtensionsLedger {
    fn stroke_paint(&self) -> std::rc::Rc<SysRecordStrokePaint>;
}

impl ExtensionsLedger for Ledger {
    fn stroke_paint(&self) -> std::rc::Rc<SysRecordStrokePaint> {
        self.read::<SysRecordStrokePaint>()
    }
}
