use core::{collections::game_state::StateOwnerships, system::system_game_state::IState};
use std::{collections::HashMap, fmt::Display, thread::panicking};

use macro_state_serialize::global_state_serialize;
use serde::{Deserialize, Serialize};

#[global_state_serialize]
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
impl IState for StateTeamAssignments {
    fn id() -> i32 {
        00988
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Host
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub enum Teams {
    Red,
    Blue,
}
impl Teams {
    pub fn convert_dir(&self, x_diff: i32, z_diff: i32) -> (i32, i32) {
        match self {
            Teams::Red => (x_diff, z_diff),
            Teams::Blue => (-x_diff, -z_diff),
        }
    }
    pub fn on_side(&self, _: i32, z: i32) -> bool {
        match self {
            Teams::Red => return z <= 1,
            Teams::Blue => return z >= 2,
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
