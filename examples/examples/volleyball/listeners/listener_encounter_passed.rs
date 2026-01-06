use crate::game_events::GameEvents;
use crate::state::host::state_currency::StateCurrency;
use crate::state::host::state_exploration::StateExploration;
use core::collections::event_queue;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use ecs_event::global_ecs_system_event_reciever;
use hecs::World;
use system_component_default_gameplay::ecs_event_reciever::{EventReciever, InstanceLimiter};
use system_component_default_gameplay::world_context::WorldContext;

#[derive(Default)]
#[global_ecs_system_event_reciever(GameEvents)]
pub struct ECSSystemGamePointScored {}

impl InstanceLimiter for ECSSystemGamePointScored {
    fn is_enabled(&mut self, _: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        NetworkModes::all_host()
    }
}
impl EventReciever<GameEvents> for ECSSystemGamePointScored {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut WorldContext, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::EncounterPassed => {
                // log
                println!("Encounter Passed");

                // claim rewards
                game_state.edit::<StateCurrency>(|x| {
                    x.currency += 100;
                });

                // request leave room
                event_queue.enqueue_event(GameEvents::DidEncounterPass);
            }
            _ => {}
        }
    }
}
