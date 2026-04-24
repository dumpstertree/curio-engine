use crate::exploration::exploration_path::RoomTypes;
use crate::state::host::state_exploration::StateExploration;
use crate::state::peer::state_peer_input_mode::InputModes;
use crate::state::peer::state_peer_input_mode::StatePeerInputMode;
use curio_core::built_in::record::sys_record_input::SysRecordInput;
use curio_core::{
    collections::{event_queue::EventQueue, game_state::Ledger},
    collections::network_modes::NetworkModes
};
use gameplay::context_3d::Context3D;
use gameplay::traits::habit::Habit;
use gameplay::traits::scope::Scope;
use habit::habit;

#[habit]
pub struct Instance {}
impl Scope for Instance {
    fn is_enabled(&mut self, game_state: &mut Ledger) -> bool {
        game_state
            .get::<StateExploration>()
            .exploration
            .get_cur_room()
            .room_type
            == RoomTypes::Combat
    }
    fn run_on_instance(&mut self, _game_state: &mut Ledger) -> Vec<NetworkModes> {
        NetworkModes::all_peer()
    }
}
impl Habit for Instance {
    fn enable(&mut self, _: &mut Ledger, _: &mut Context3D, _: &mut EventQueue) {}
    fn tick(&mut self, game_state: &mut Ledger, _: &mut Context3D, _events: &mut EventQueue) {
        let state_input = game_state.get::<SysRecordInput>();

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
