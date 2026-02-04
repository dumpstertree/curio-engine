use std::fmt::Display;

use curio_core::{Vector2Int, Vector3};

use crate::state::state_teams::Teams;

pub struct GameBoard {}
impl GameBoard {
    pub fn get_serving_tile(for_team: &Teams) -> (i32, i32) {
        match for_team {
            Teams::Red => (1, 0),
            Teams::Blue => (2, 3),
        }
    }

    pub fn do_move(team: &Teams, tile: &(i32, i32), direction: &Directions) -> (i32, i32) {
        // get the standard offset based on the direction
        let offset = match direction {
            Directions::Forward => (0, 1),
            Directions::Back => (0, -1),
            Directions::Left => (-1, 0),
            Directions::Right => (1, 0),
        };

        // conver the direction based on the team that is moving
        let converted_offset = team.convert_dir(offset.0, offset.1);

        // get the new tile based on the original tile and the converted offset
        (tile.0 + converted_offset.0, tile.1 + converted_offset.1)
    }
    pub fn can_move(team: &Teams, tile: &(i32, i32), direction: Directions) -> bool {
        // get the standard offset based on the direction
        let offset = match direction {
            Directions::Forward => (0, 1),
            Directions::Back => (0, -1),
            Directions::Left => (-1, 0),
            Directions::Right => (1, 0),
        };

        // conver the direction based on the team that is moving
        let converted_offset = team.convert_dir(offset.0, offset.1);

        // get the new tile based on the original tile and the converted offset
        let new_tile = (tile.0 + converted_offset.0, tile.1 + converted_offset.1);

        // get the bounds for the given team
        let min = Self::get_bounds_min_for_team(team);
        let max = Self::get_bounds_max_for_team(team);

        // make sure its in bounds
        let in_x = new_tile.0 <= max.x && new_tile.0 >= min.x;
        let in_z = new_tile.1 <= max.y && new_tile.1 >= min.y;

        // return if we are in both x and z
        return in_x && in_z;
    }

    pub fn get_tiles() -> Vec<Vector2Int> {
        let mut output = Vec::new();
        let min = Self::get_bounds_min();
        let max = Self::get_bounds_max();
        for x in min.x..(max.x + 1) {
            for y in min.y..(max.y + 1) {
                output.push(Vector2Int::new(x, y));
            }
        }
        output
    }
    pub fn get_back_corners_for_team(team: &Teams) -> Vec<Vector2Int> {
        match team {
            Teams::Red => vec![Vector2Int::new(0, 0), Vector2Int::new(3, 0)],
            Teams::Blue => vec![Vector2Int::new(0, 3), Vector2Int::new(3, 3)],
        }
    }
    pub fn get_front_corners_for_team(team: &Teams) -> Vec<Vector2Int> {
        match team {
            Teams::Red => vec![Vector2Int::new(0, 1), Vector2Int::new(3, 1)],
            Teams::Blue => vec![Vector2Int::new(0, 2), Vector2Int::new(3, 2)],
        }
    }
    pub fn get_tiles_for_team(team: &Teams) -> Vec<Vector2Int> {
        let mut output = Vec::new();
        let min = Self::get_bounds_min_for_team(&team);
        let max = Self::get_bounds_max_for_team(&team);
        for x in min.x..(max.x + 1) {
            for y in min.y..(max.y + 1) {
                output.push(Vector2Int::new(x, y));
            }
        }
        output
    }
    pub fn get_bounds_min() -> Vector2Int {
        Vector2Int::new(-1, -1)
    }
    pub fn get_bounds_max() -> Vector2Int {
        Vector2Int::new(4, 4)
    }
    pub fn get_bounds_min_for_team(for_team: &Teams) -> Vector2Int {
        match for_team {
            Teams::Red => Vector2Int::new(0, 0),
            Teams::Blue => Vector2Int::new(0, 2),
        }
    }
    pub fn get_bounds_max_for_team(for_team: &Teams) -> Vector2Int {
        match for_team {
            Teams::Red => Vector2Int::new(3, 1),
            Teams::Blue => Vector2Int::new(3, 3),
        }
    }
    pub fn get_world_position(x: i32, z: i32) -> Vector3 {
        let fl_z = 3.0;
        let bl_z = 7.0;
        let ob_z = 11.0;
        let row_00 = 6.8;
        let row_0 = 3.8;
        let row_1 = 1.4;
        let row_2 = -1.4;
        let row_3 = -3.8;
        let row_33 = -6.8;
        let p = [
            [(row_00, -ob_z), (row_0, -ob_z), (row_1, -ob_z), (row_2, -ob_z), (row_3, -ob_z), (row_33, -ob_z)], // out of bounds
            [(row_00, -bl_z), (row_0, -bl_z), (row_1, -bl_z), (row_2, -bl_z), (row_3, -bl_z), (row_33, -bl_z)], // red_back
            [(row_00, -fl_z), (row_0, -fl_z), (row_1, -fl_z), (row_2, -fl_z), (row_3, -fl_z), (row_33, -fl_z)], // red_front
            [(row_00, fl_z), (row_0, fl_z), (row_1, fl_z), (row_2, fl_z), (row_3, fl_z), (row_33, fl_z)],       // blue_front
            [(row_00, bl_z), (row_0, bl_z), (row_1, bl_z), (row_2, bl_z), (row_3, bl_z), (row_33, bl_z)],       // blue_back
            [(row_00, ob_z), (row_0, ob_z), (row_1, ob_z), (row_2, ob_z), (row_3, ob_z), (row_33, ob_z)],       // blue out of bounds
        ];

        let x = x + 1;
        let z = z + 1;

        let x = x.max(0);
        let x = x.min(6);
        let z = z.max(0);
        let z = z.min(6);
        Vector3::new(p[z as usize][x as usize].0, 0.0, p[z as usize][x as usize].1)
    }
}

#[derive(Clone, Debug)]
pub enum Directions {
    Forward,
    Back,
    Left,
    Right,
}
impl Display for Directions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Directions::Forward => f.write_str("Forward"),
            Directions::Back => f.write_str("Back"),
            Directions::Left => f.write_str("Left"),
            Directions::Right => f.write_str("Right"),
        }
    }
}
