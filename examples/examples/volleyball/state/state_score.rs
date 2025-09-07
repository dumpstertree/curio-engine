use core::{collections::game_state::StateOwnerships, system::system_game_state::IState};
use std::collections::HashMap;

use macro_state_serialize::global_state_serialize;

use crate::state::state_teams::Teams;

#[global_state_serialize]
pub struct StateScore {
    pub all_scores: HashMap<Teams, i32>,
}
impl IState for StateScore {
    fn id() -> i32 {
        90809
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Host
    }
}
