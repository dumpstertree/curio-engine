use curio_core::{collections::state_ownerships::StateOwnerships, system::system_game_state::RecordCommon};

use record_serializable::record_serializable;

use crate::exploration::exploration_path::Exploration;

#[derive(Hash, PartialEq, Eq)]
#[record_serializable]
pub struct StateExploration {
    pub exploration: Exploration,
    pub is_selecting_next: bool,
}
impl RecordCommon for StateExploration {
    fn id() -> i32 {
        827364
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Host
    }

    fn default_box() -> Box<dyn RecordCommon>
    where
        Self: Sized + Default + 'static,
    {
        Box::new(Self::default())
    }
}
