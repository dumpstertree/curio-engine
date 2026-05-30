use std::sync::{Arc, OnceLock};

use curio_core::{Color, FieldState, RecordCommon, RecordId, RecordOverride, TextureAsset};

static SYS_RECORD_ID: OnceLock<i32> = OnceLock::new();

#[derive(Default, Hash, PartialEq, Eq, Clone)]
pub struct SysRecordSkybox {
    pub skybox: SkyboxTypes,
}
impl SysRecordSkybox {}
impl RecordCommon for SysRecordSkybox {
    fn name(&self) -> String {
        String::from("Skybox")
    }
    fn id() -> i32 {
        *SYS_RECORD_ID.get_or_init(|| RecordId::of::<SysRecordSkybox>())
    }
}

#[derive(Default, Clone, Hash, PartialEq, Eq)]
pub enum SkyboxTypes {
    #[default]
    Defualt,
    Color(Color),
    CubeMap(Arc<TextureAsset>),
}
impl RecordOverride for SysRecordSkybox {
    fn apply(&mut self, _: &str, _: &str) {}
    fn get_state(&self) -> Vec<FieldState> {
        match &self.skybox {
            SkyboxTypes::Defualt => vec![FieldState::new("type", "default")],
            SkyboxTypes::Color(color) => vec![FieldState::new("type", "default"), FieldState::new("color", color)],
            SkyboxTypes::CubeMap(_) => vec![FieldState::new("type", "cubemap")],
        }
    }
}
