use core::{
    collections::{state_ownerships::StateOwnerships, vector2_int::Vector2Int},
    system::system_game_state::IState,
};

use macro_state_serialize::global_state_serialize;
use serde::{Deserialize, Serialize};

use crate::listeners::listener_start_encounter::Encounter;

#[derive(Hash, PartialEq, Eq)]
#[global_state_serialize]
pub struct StateEncounter {
    pub encounter: Encounter,
}
impl IState for StateEncounter {
    fn id() -> i32 {
        990249234
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Instance
    }

    fn default_box() -> Box<dyn IState>
    where
        Self: Sized + Default + 'static,
    {
        Box::new(Self::default())
    }
}
