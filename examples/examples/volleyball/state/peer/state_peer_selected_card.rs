use curio_core::{collections::state_ownerships::StateOwnerships, system::system_game_state::IState};

use record_serializable::record_serializable;
#[derive(Hash, PartialEq, Eq)]
#[record_serializable]
pub struct StatePeerSelectedCards {
    pub index: i32,
}
impl IState for StatePeerSelectedCards {
    fn id() -> i32 {
        89375938
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Instance
    }
}
