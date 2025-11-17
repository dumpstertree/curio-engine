use crate::state::peer::state_peer_input_mode::InputModes;
use crate::state::peer::state_peer_input_mode::StatePeerInputMode;
use built_in_state::state_input::InputState;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
};
use ecs_system::global_ecs_system;
use hecs::World;

#[global_ecs_system]
pub struct ECSSystemTurnEnd {}
impl ECSSystemEventless for ECSSystemTurnEnd {
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalPeer, NetworkModes::OnlinePeer]
    }
    fn is_enabled(&mut self, game_state: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn enable(&mut self, _: &mut GameState, _: &mut World, _: &mut EventQueue) {}
    fn tick(&mut self, game_state: &mut GameState, _: &mut World, events: &mut EventQueue) {
        let state_input = game_state.get_value2::<InputState>();

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
