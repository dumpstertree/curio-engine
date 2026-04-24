use curio_core::{collections::state_ownerships::StateOwnerships, system::system_game_state::RecordCommon};
use record_serializable::record_serializable;
use std::hash::Hash;

#[derive(Hash, PartialEq, Eq)]
#[record_serializable]
pub struct StateCurrency {
    pub currency: i32,
}
impl RecordCommon for StateCurrency {
    fn id() -> i32 {
        901812999
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Host
    }
}
