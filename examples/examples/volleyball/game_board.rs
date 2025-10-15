use core::collections::{vector2_int::Vector2Int, vector3::Vector3};

use crate::state::state_teams::Teams;

pub struct GameBoard {}
impl GameBoard {
    pub fn get_serving_tile(for_team: &Teams) -> (i32, i32) {
        match for_team {
            Teams::Red => (1, 0),
            Teams::Blue => (2, 5),
        }
    }
    pub fn get_bounds_min(for_team: &Teams) -> Vector2Int {
        match for_team {
            Teams::Red => Vector2Int::new(0, 1),
            Teams::Blue => Vector2Int::new(0, 3),
        }
    }
    pub fn get_bounds_max(for_team: &Teams) -> Vector2Int {
        match for_team {
            Teams::Red => Vector2Int::new(3, 2),
            Teams::Blue => Vector2Int::new(3, 4),
        }
    }
    pub fn get_world_position(x: i32, z: i32) -> Vector3 {
        let fl_z = 3.0;
        let bl_z = 7.0;
        let sl_z = 11.0;
        let row_0 = 3.8;
        let row_1 = 1.4;
        let row_2 = -1.4;
        let row_3 = -3.8;
        let p = [
            [(row_0, -sl_z), (row_1, -sl_z), (row_2, -sl_z), (row_3, -sl_z)], // red_serving
            [(row_0, -bl_z), (row_1, -bl_z), (row_2, -bl_z), (row_3, -bl_z)], // red_back
            [(row_0, -fl_z), (row_1, -fl_z), (row_2, -fl_z), (row_3, -fl_z)], // red_front
            [(row_0, fl_z), (row_1, fl_z), (row_2, fl_z), (row_3, fl_z)],     // blue_front
            [(row_0, bl_z), (row_1, bl_z), (row_2, bl_z), (row_3, bl_z)],     // blue_back
            [(row_0, sl_z), (row_1, sl_z), (row_2, sl_z), (row_3, sl_z)],     // blue_serving
        ];

        let x = x.max(0);
        let z = z.max(0);
        let x = x.min(3);
        let z = z.min(5);
        Vector3::new(p[z as usize][x as usize].0, 0.0, p[z as usize][x as usize].1)
    }
}
