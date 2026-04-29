use curio_core::{collections::state_ownerships::StateOwnerships, system::system_game_state::RecordCommon};
use record_serializable::record_serializable;
use std::hash::Hash;

#[derive(Hash, PartialEq, Eq)]
#[record_serializable(ownership = StateOwnerships::Host)]
pub struct StateCurrency {
    pub currency: i32,
}
