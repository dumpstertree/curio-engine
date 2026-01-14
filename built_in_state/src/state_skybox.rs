use curio_core::{collections::color::Color, io::texture_asset::TextureAsset, system::system_game_state::IState};
use macro_state::global_state;
use std::sync::Arc;

#[derive(Hash, Eq)]
#[global_state]
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
