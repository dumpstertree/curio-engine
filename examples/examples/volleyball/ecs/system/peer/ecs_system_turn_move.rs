use built_in_state::state_input::InputState;
use ecs_system::global_ecs_system;
use hecs::World;

use core::{
    collections::{
        event_queue::{self, EventQueue},
        game_state::GameState,
    },
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
};

use crate::{
    game_events::GameEvents,
    state::{state_energy::StateEnergy, state_turn::StateTurn},
};

#[global_ecs_system]
pub struct ECSSystemTurnMove {}
impl ECSSystemEventless for ECSSystemTurnMove {
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalPeer, NetworkModes::OnlinePeer]
    }
    fn is_enabled(&mut self, game_state: &mut GameState, _: &mut World) -> bool {
        game_state.get_value2::<StateTurn>().active_instance_id == game_state.instance_id
    }
    fn tick(&mut self, game_state: &mut GameState, _: &mut World, event_queue: &mut EventQueue) {
        // get states
        let state_input = game_state.get_value2::<InputState>();

        // get inputs from mapping
        let move_forward = state_input.mapped[0]
            .get_button_or_default("move_forward")
            .went_up;
        let move_back: bool = state_input.mapped[0]
            .get_button_or_default("move_back")
            .went_up;
        let move_left = state_input.mapped[0]
            .get_button_or_default("move_left")
            .went_up;
        let move_right: bool = state_input.mapped[0]
            .get_button_or_default("move_right")
            .went_up;

        // if any movement detected
        if move_forward || move_back || move_left || move_right {
            if move_forward {
                event_queue.enqueue_event(GameEvents::RequestMoveZPos(game_state.instance_id));
            }

            if move_back {
                event_queue.enqueue_event(GameEvents::RequestMoveZNeg(game_state.instance_id));
            }

            if move_left {
                event_queue.enqueue_event(GameEvents::RequestMoveXNeg(game_state.instance_id));
            }

            if move_right {
                event_queue.enqueue_event(GameEvents::RequestMoveXPos(game_state.instance_id));
            }
        }
    }
}
