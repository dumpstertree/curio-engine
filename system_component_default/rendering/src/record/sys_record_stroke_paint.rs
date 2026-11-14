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
    pub flow: f32,
    pub bristle: f32,
}

impl SysRecordStrokePaint {
    // defaults mirror RenderFeaturePostProcessPainterly::new()'s constructor
    pub fn default() -> SysRecordStrokePaint {
        SysRecordStrokePaint {
            size: 22.0, // long-axis daub in flat areas
            size_min: 0.35, // busiest areas keep ~a third of that length
            mag_scale: 45.0, // slope (luminance per px) sensitivity: dirDeriv divides out the probe displacement, so a grout jump of ~0.4 over 15px stands at ~0.026/px and this pins it at full contrast
            elongation: 2.6, // anisotropic neighbourhoods (a real edge) carry the full stretch
            aspect_min: 1.2, // no edge to follow: near-round daub
            alpha: 1.0,
            feather: 0.45, // soft edge fraction of the analytic brush
            keep: 1.0, // every slot draws; lower it thins the paint
            jitter: 0.45,
            wobble: 0.25,
            seed: 0.0,
            flow: 1.0, // daub shape follows the tensor's anisotropy
            bristle: 0.75, // brush mask drives the stamp alpha
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
            "flow" => { self.flow = val }
            "bristle" => { self.bristle = val }
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
            FieldState::new("flow", self.flow),
            FieldState::new("bristle", self.bristle),
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
