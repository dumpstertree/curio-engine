use core::collections::vector2_int::Vector2Int;

#[derive(Default)]
pub struct ComponentGameBoardTile {
    pub tile: Vector2Int,
}
impl ComponentGameBoardTile {
    pub fn set_tile(mut self, tile: Vector2Int) -> Self {
        self.tile = tile;
        self
    }
}
