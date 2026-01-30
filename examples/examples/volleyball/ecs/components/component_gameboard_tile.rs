use curio_core::collections::vector2_int::Vector2Int;
use gameplay::traits::field_override::FieldOverride;
use macro_component::facet;

#[facet]
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
