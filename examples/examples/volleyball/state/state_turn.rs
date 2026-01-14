use curio_core::{collections::state_ownerships::StateOwnerships, system::system_game_state::IState};

use macro_state_serialize::global_state_serialize;

use crate::state::state_teams::Teams;

#[derive(Hash, PartialEq, Eq)]
#[global_state_serialize]
pub struct StateTurn {
    pub active_instance_id: Teams,
}
impl IState for StateTurn {
    fn id() -> i32 {
        0005
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Host
    }
}
