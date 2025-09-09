use core::collections::{vector2_int::Vector2Int, vector3::Vector3};

use crate::state::state_teams::Teams;

pub struct GameBoard {}
impl GameBoard {
    pub fn get_serving_tile(for_team: &Teams) -> (i32, i32) {
        match for_team {
            Teams::Red => (1, -1),
            Teams::Blue => (2, 4),
        }
    }
    pub fn get_bounds_min(for_team: &Teams) -> Vector2Int {
        match for_team {
            Teams::Red => Vector2Int::new(0, 0),
            Teams::Blue => Vector2Int::new(0, 2),
        }
    }
    pub fn get_bounds_max(for_team: &Teams) -> Vector2Int {
        match for_team {
            Teams::Red => Vector2Int::new(3, 1),
            Teams::Blue => Vector2Int::new(3, 3),
        }
    }
    pub fn get_world_position(x: i32, z: i32) -> Vector3 {
        let z = z as f32;

        let num_tiles_x = 4.0;
        let num_tiles_z = 4.0;
        let x = num_tiles_x - x as f32;
        let tile_size = 3.0;

        // get max size
        let max_x = num_tiles_x * tile_size;
        let max_z = num_tiles_z * tile_size;

        // get start point
        let start_x = -max_x / 2.0;
        let start_z = -max_z / 2.0;

        Vector3::new(start_x + x * tile_size, 0.0, start_z + z * tile_size)
    }
}
