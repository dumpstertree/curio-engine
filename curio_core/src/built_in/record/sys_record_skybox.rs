use std::sync::Arc;

use crate::{io::texture_asset::TextureAsset, system::system_game_state::IState, Color};

#[derive(Default, Hash, PartialEq, Eq, Clone)]
pub struct SysRecordSkybox {
    pub skybox: SkyboxTypes,
}
impl SysRecordSkybox {}
impl IState for SysRecordSkybox {
    fn id() -> i32 {
        9806666
    }
}

#[derive(Default, Clone, Hash, PartialEq, Eq)]
pub enum SkyboxTypes {
    #[default]
    Defualt,
    Color(Color),
    CubeMap(Arc<TextureAsset>),
}
