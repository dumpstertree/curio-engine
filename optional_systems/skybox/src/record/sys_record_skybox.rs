use curio_core::{Color, FieldState, RecordOverride, RecordScope, TextureAsset};
use record::record;
use std::sync::Arc;

#[record(name = "Skybox", ownership = RecordScope::Instance)]
pub struct SysRecordSkybox {
    pub skybox: SkyboxTypes,
}
impl SysRecordSkybox {}

#[derive(Default, Clone, Hash, PartialEq, Eq)]
pub enum SkyboxTypes {
    #[default]
    Defualt,
    Color(Color),
    CubeMap(Arc<TextureAsset>),
}
impl RecordOverride for SysRecordSkybox {
    fn set_state(&mut self, _: &str, _: &str) {}
    fn get_state(&self) -> Vec<FieldState> {
        match &self.skybox {
            SkyboxTypes::Defualt => vec![FieldState::new("type", "default")],
            SkyboxTypes::Color(color) => vec![FieldState::new("type", "default"), FieldState::new("color", color)],
            SkyboxTypes::CubeMap(_) => vec![FieldState::new("type", "cubemap")],
        }
    }
}
