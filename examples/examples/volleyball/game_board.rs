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
        let fl_z = 3.0;
        let bl_z = 7.0;
        let row_0 = 3.8;
        let row_1 = 1.4;
        let row_2 = -1.4;
        let row_3 = -3.8;
        let p = [
            [(row_0, -bl_z), (row_1, -bl_z), (row_2, -bl_z), (row_3, -bl_z)], // red_back
            [(row_0, -fl_z), (row_1, -fl_z), (row_2, -fl_z), (row_3, -fl_z)], // red_front
            [(row_0, fl_z), (row_1, fl_z), (row_2, fl_z), (row_3, fl_z)],     // blue_front
            [(row_0, bl_z), (row_1, bl_z), (row_2, bl_z), (row_3, bl_z)],     // blue_back
        ];

        // let z = z as f32;

        // let z_offset = 1.0;
        // let num_tiles_x = 4.0;
        // let num_tiles_z = 4.0;
        // let x = num_tiles_x - x as f32;
        // let tile_size_x = 1.75;
        // let tile_size_z = 3.0;

        // // get max size
        // let max_x = num_tiles_x * tile_size_x;
        // let max_z = num_tiles_z * tile_size_z + z_offset;

        // // get start point
        // let start_x = -max_x / 2.0;
        // let start_z = -max_z / 2.0;

        // let mut o = z_offset;
        // if z < 0.0 {
        //     o = -z_offset
        // }

        // Vector3::new(start_x + x * tile_size_x, 0.0, o + start_z + z * tile_size_z)
        let x = x.max(0);
        let z = z.max(0);
        let x = x.min(3);
        let z = z.min(3);
        Vector3::new(p[z as usize][x as usize].0, 0.0, p[z as usize][x as usize].1)
    }
}
