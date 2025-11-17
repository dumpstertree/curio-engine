use core::{collections::game_state::StateOwnerships, system::system_game_state::IState};

use macro_state_serialize::global_state_serialize;
#[derive(Hash, PartialEq, Eq)]
#[global_state_serialize]
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
