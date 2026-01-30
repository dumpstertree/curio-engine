use crate::exploration::exploration_path::RoomTypes;
use crate::state::host::state_exploration::StateExploration;
use crate::state::peer::state_peer_input_mode::InputModes;
use crate::state::peer::state_peer_input_mode::StatePeerInputMode;
use curio_core::built_in::record::state_input::InputState;
use curio_core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use ecs_system::habit;
use gameplay::context_3d::Context3D;
use gameplay::traits::habit::Habit;
use gameplay::traits::scope::Scope;

#[habit]
pub struct Instance {}
impl Scope for Instance {
    fn is_enabled(&mut self, game_state: &mut GameState) -> bool {
        game_state
            .get::<StateExploration>()
            .exploration
            .get_cur_room()
            .room_type
            == RoomTypes::Combat
    }
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<NetworkModes> {
        NetworkModes::all_peer()
    }
}
impl Habit for Instance {
    fn enable(&mut self, _: &mut GameState, _: &mut Context3D, _: &mut EventQueue) {}
    fn tick(&mut self, game_state: &mut GameState, _: &mut Context3D, events: &mut EventQueue) {
        let state_input = game_state.get::<InputState>();

        game_state.edit::<StatePeerInputMode>(|x| {
            if state_input.mapped[0]
                .get_button_or_default("card_mode")
                .is_down
            {
                x.mode = InputModes::Manuever;
            } else {
                x.mode = InputModes::Move;
            }
        });
    }
}
