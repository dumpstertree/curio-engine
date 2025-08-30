use crate::game_events::GameEvents;
use crate::state::state_turn::StateTurn;
use built_in_state::state_input::InputState;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
};
use ecs_system::global_ecs_system;
use hecs::World;

#[global_ecs_system]
pub struct ECSSystemTurnEnd {}
impl ECSSystemEventless for ECSSystemTurnEnd {
    fn is_enabled(&mut self, game_state: &mut GameState, _: &mut World) -> bool {
        game_state.get_value2::<StateTurn>().active_instance_id == game_state.instance_id
    }
    fn tick(&mut self, game_state: &mut GameState, _: &mut World, events: &mut EventQueue) {
        // get input
        let state_input = game_state.get_value2::<InputState>();

        // guard - input for next turn
        let input_next = state_input.mapped[0]
            .get_button_or_default("turn_end")
            .went_up;
        if !input_next {
            return;
        }

        println!("send turn end");
        // send event to end turn
        events.enqueue_event(GameEvents::TurnEnd(game_state.instance_id));
    }
}
