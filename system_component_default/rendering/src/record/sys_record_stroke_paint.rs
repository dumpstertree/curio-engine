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
    pub color_jitter: f32,
    pub world_size: f32,
    pub world_cell: f32,
    pub drift: f32,
    pub life: f32,
    pub mass_mult: f32,
    pub mass_alpha: f32,
    pub mid_mult: f32,
    pub mid_alpha: f32,
    pub fine_mult: f32,
    pub fine_alpha: f32,
    pub contrast_split_low: f32,
    pub contrast_split_high: f32,
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
            color_jitter: 0.16, // forced per-channel deviation from the guide colour (minimum 0.6 * this), so stamps are distinguishable inside flat kuwahara blocks
            world_size: 0.0, // 0 = daub long axis stays in px; > 0 = WORLD units through the focal length, so distant paint shrinks with the scene
            world_cell: 0.25, // world-space lattice the unprojected ray hits snap to: the anchor's identity, ~25 cm cells at reference scale
            drift: 0.02, // sinusoidal sway inside a stroke's own lattice cell, as a fraction of that cell: a hair of motion so the paint breathes instead of sitting dead flat
            life: 4.0, // mean birth/death cycle in seconds (0 = hard pops kept): each stroke fades up and down on a phase keyed on its own identity, so what the camera reveals and retires lands mid-fade instead of popping at full alpha
            // three contrast-gated layers over one draw, each with its OWN full
            // grid: slots [0,4096) are mass (long-axis mult x3 of the base size,
            // lattice scaled to match, painted UNDER everything), [4096,8192) mid,
            // the rest detail
            mass_mult: 3.0, // mass layer long axis as a mult of the base size
            mass_alpha: 0.45, // masses stay quiet so they read as underpainting
            mid_mult: 1.5, // mid-tone layer long-axis mult
            mid_alpha: 0.8,
            fine_mult: 1.0, // detail layer keeps the existing daub regime
            fine_alpha: 1.0,
            contrast_split_low: 0.35, // mass paints only where local contrast is below this: broad strokes stay in FLAT neighbourhoods (the research fix for large daubs landing on detailed zones)
            contrast_split_high: 0.7, // detail paints only above this; mid fills the band between the two splits
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
            "color_jitter" => { self.color_jitter = val }
            "world_size" => { self.world_size = val }
            "world_cell" => { self.world_cell = val }
            "drift" => { self.drift = val }
            "life" => { self.life = val }
            "mass_mult" => { self.mass_mult = val }
            "mass_alpha" => { self.mass_alpha = val }
            "mid_mult" => { self.mid_mult = val }
            "mid_alpha" => { self.mid_alpha = val }
            "fine_mult" => { self.fine_mult = val }
            "fine_alpha" => { self.fine_alpha = val }
            "contrast_split_low" => { self.contrast_split_low = val }
            "contrast_split_high" => { self.contrast_split_high = val }
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
            FieldState::new("color_jitter", self.color_jitter),
            FieldState::new("world_size", self.world_size),
            FieldState::new("world_cell", self.world_cell),
            FieldState::new("drift", self.drift),
            FieldState::new("life", self.life),
            FieldState::new("mass_mult", self.mass_mult),
            FieldState::new("mass_alpha", self.mass_alpha),
            FieldState::new("mid_mult", self.mid_mult),
            FieldState::new("mid_alpha", self.mid_alpha),
            FieldState::new("fine_mult", self.fine_mult),
            FieldState::new("fine_alpha", self.fine_alpha),
            FieldState::new("contrast_split_low", self.contrast_split_low),
            FieldState::new("contrast_split_high", self.contrast_split_high),
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
