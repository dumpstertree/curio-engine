use std::sync::Arc;

use crate::{system::system_game_state::RecordCommon, Color, TextureAsset};

#[derive(Default, Hash, PartialEq, Eq, Clone)]
pub struct SysRecordSkybox {
    pub skybox: SkyboxTypes,
}
impl SysRecordSkybox {}
impl RecordCommon for SysRecordSkybox {
    fn id() -> i32 {
        112
    }
}

#[derive(Default, Clone, Hash, PartialEq, Eq)]
pub enum SkyboxTypes {
    #[default]
    Defualt,
    Color(Color),
    CubeMap(Arc<TextureAsset>),
}
