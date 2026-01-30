use std::sync::Arc;

use crate::{collections::color::Color, io::texture_asset::TextureAsset, system::system_game_state::IState};

#[derive(Default, Hash, PartialEq, Eq, Clone)]
pub struct StateSkybox {
    pub skybox: SkyboxTypes,
}
impl StateSkybox {}
impl IState for StateSkybox {
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
