use curio_core::{collections::state_ownerships::StateOwnerships, system::system_game_state::IState};

use record_serializable::record_serializable;
#[derive(Hash, PartialEq, Eq)]
#[record_serializable]
pub struct StatePositionBall {
    pub row: i32,
    pub column: i32,
}
impl IState for StatePositionBall {
    fn id() -> i32 {
        0002
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Host
    }
}
