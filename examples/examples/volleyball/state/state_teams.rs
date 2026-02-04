use curio_core::{collections::state_ownerships::StateOwnerships, random::Random, system::system_game_state::IState};
use std::{collections::HashMap, fmt::Display, hash::Hash};

use record_serializable::record_serializable;
use serde::{Deserialize, Serialize};

use crate::game_board::GameBoard;

#[derive(PartialEq, Eq)]
#[record_serializable]
pub struct StateTeamAssignments {
    pub team_assignments: HashMap<Teams, Vec<i32>>,
}
impl StateTeamAssignments {
    pub fn team_for(&self, player_id: &i32) -> Option<Teams> {
        for x in &self.team_assignments {
            if x.1.contains(&player_id) {
                return Some(x.0.clone());
            }
        }
        None
    }
}
impl Hash for StateTeamAssignments {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut axis_keys: Vec<&Teams> = self.team_assignments.keys().collect();
        axis_keys.sort();
        axis_keys.len().hash(state);
        for k in axis_keys {
            k.hash(state);
            self.team_assignments.get(k).unwrap().hash(state);
        }
    }
}
impl IState for StateTeamAssignments {
    fn id() -> i32 {
        00988
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Host
    }
}

#[derive(Default, PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum Teams {
    #[default]
    Red,
    Blue,
}
impl Teams {
    pub fn all() -> Vec<Teams> {
        vec![Teams::Red, Teams::Blue]
    }
    pub fn random() -> Teams {
        if Random::random_bool() { Teams::Red } else { Teams::Blue }
    }
    pub fn convert_dir(&self, x_diff: i32, z_diff: i32) -> (i32, i32) {
        match self {
            Teams::Red => (x_diff, z_diff),
            Teams::Blue => (-x_diff, -z_diff),
        }
    }
    pub fn is_on_side(&self, _: i32, z: i32) -> bool {
        match self {
            Teams::Red => return z <= GameBoard::get_bounds_max_for_team(&Teams::Red).y,
            Teams::Blue => return z >= GameBoard::get_bounds_min_for_team(&Teams::Blue).y,
        }
    }
    pub fn is_out_of_bounds(&self, x: i32, z: i32) -> bool {
        match self {
            Teams::Red => {
                if x < GameBoard::get_bounds_min_for_team(&Teams::Red).x || x < GameBoard::get_bounds_max_for_team(&Teams::Red).x || z > GameBoard::get_bounds_max_for_team(&Teams::Red).y {
                    return true;
                } else {
                    return false;
                }
            }
            Teams::Blue => {
                if x < GameBoard::get_bounds_min_for_team(&Teams::Blue).x || x < GameBoard::get_bounds_max_for_team(&Teams::Blue).x || z < GameBoard::get_bounds_min_for_team(&Teams::Blue).y {
                    return true;
                } else {
                    return false;
                }
            }
        }
    }
    pub fn next_team(&self) -> Self {
        match self {
            Teams::Red => Teams::Blue,
            Teams::Blue => Teams::Red,
        }
    }
}
impl Display for Teams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Teams::Red => write!(f, "Red"),
            Teams::Blue => write!(f, "Blue"),
        }
    }
}
