use curio_core::{FieldState, Ledger, RecordOverride, RecordScope};
use record::record;

#[record(name = "PostProcess", ownership = RecordScope::Instance)]
pub struct SysRecordPostProcess {
    pub radius: f32,
    pub strength: f32,
    pub sigma: f32,
    pub tau: f32,
    pub dither_mode: f32,
    pub dither_amount: f32,
    pub seed: f32,
    pub depth_strength: f32,
    pub temporal: f32,
}

impl SysRecordPostProcess {
    // defaults mirror RenderFeaturePostProcessKuwahara::new()'s constructor
    pub fn default() -> SysRecordPostProcess {
        SysRecordPostProcess {
            radius: 4.0,
            strength: 1.0,
            sigma: 1.5,
            tau: 3.0,
            dither_mode: 0.0,
            dither_amount: 0.0,
            seed: 0.0,
            depth_strength: 40.0,
            temporal: 0.0,
        }
    }
}

impl RecordOverride for SysRecordPostProcess {
    // lets the debug GUI / save-state tooling push values straight in
    fn set_state(&mut self, field: &str, val: &str) {
        let Some(val) = val.parse::<f32>().ok() else { return };
        match field {
            "radius" => { self.radius = val }
            "strength" => { self.strength = val }
            "sigma" => { self.sigma = val }
            "tau" => { self.tau = val }
            "dither_mode" => { self.dither_mode = val }
            "dither_amount" => { self.dither_amount = val }
            "seed" => { self.seed = val }
            "depth_strength" => { self.depth_strength = val }
            "temporal" => { self.temporal = val }
            _ => {}
        }
    }
    fn get_state(&self) -> Vec<FieldState> {
        vec![
            FieldState::new("radius", self.radius),
            FieldState::new("strength", self.strength),
            FieldState::new("sigma", self.sigma),
            FieldState::new("tau", self.tau),
            FieldState::new("dither_mode", self.dither_mode),
            FieldState::new("dither_amount", self.dither_amount),
            FieldState::new("seed", self.seed),
            FieldState::new("depth_strength", self.depth_strength),
            FieldState::new("temporal", self.temporal),
        ]
    }
}

pub trait ExtensionsLedger {
    fn post_process(&self) -> std::rc::Rc<SysRecordPostProcess>;
}

impl ExtensionsLedger for Ledger {
    fn post_process(&self) -> std::rc::Rc<SysRecordPostProcess> {
        self.read::<SysRecordPostProcess>()
    }
}
