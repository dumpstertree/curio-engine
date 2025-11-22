use core::{collections::state_ownerships::StateOwnerships, system::system_game_state::IState};
use std::fmt;

use macro_state_serialize::global_state_serialize;
use serde::{Deserialize, Serialize};

#[derive(Hash, PartialEq, Eq)]
#[global_state_serialize]
pub struct StateBallMode {
    pub mode: BallModes,
}
impl IState for StateBallMode {
    fn id() -> i32 {
        0003
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Host
    }
}
#[derive(Hash, PartialEq, Eq, Default, Clone, Serialize, Deserialize)]
pub enum BallModes {
    #[default]
    Serve,
    Bump,
    Set,
    Spike,
    Scored,
}
impl fmt::Display for BallModes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BallModes::Serve => write!(f, "Serve"),
            BallModes::Bump => write!(f, "Bump"),
            BallModes::Set => write!(f, "Set"),
            BallModes::Spike => write!(f, "Spike"),
            BallModes::Scored => write!(f, "Scored"),
        }
    }
}
