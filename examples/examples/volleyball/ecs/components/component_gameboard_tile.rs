use core::collections::vector2_int::Vector2Int;
use macro_component::global_component;
use system_component_default_gameplay::traits::field_override::FieldOverride;

#[global_component]
pub struct ComponentGameBoardTile {
    pub tile: Vector2Int,
}
impl ComponentGameBoardTile {
    pub fn set_tile(mut self, tile: Vector2Int) -> Self {
        self.tile = tile;
        self
    }
}
impl FieldOverride for ComponentGameBoardTile {
    fn apply(&mut self, _field: &str, _val: &str) {}
}
// impl Default for ComponentGameBoardTile {
//     fn default() -> Self {
//         Self { owner: None, tile: Vector2Int::zero() }
//     }
// }
