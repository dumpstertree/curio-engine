use curio_core::{collections::state_ownerships::StateOwnerships, system::system_game_state::IState};

use record_serializable::record_serializable;

use crate::exploration::exploration_path::Exploration;

#[derive(Hash, PartialEq, Eq)]
#[record_serializable]
pub struct StateExploration {
    pub exploration: Exploration,
    pub is_selecting_next: bool,
}
impl IState for StateExploration {
    fn id() -> i32 {
        827364
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Host
    }

    fn default_box() -> Box<dyn IState>
    where
        Self: Sized + Default + 'static,
    {
        Box::new(Self::default())
    }
}
