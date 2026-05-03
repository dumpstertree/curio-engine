use std::fmt;

use curio_core::StateOwnerships;
use record_serializable::record_serializable;
use serde::{Deserialize, Serialize};

#[derive(Hash, PartialEq, Eq)]
#[record_serializable(serializable,ownership = StateOwnerships::Host)]
pub struct StateBallMode {
    pub mode: BallModes,
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
