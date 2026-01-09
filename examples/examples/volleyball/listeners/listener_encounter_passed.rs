use crate::game_events::GameEvents;
use crate::state::host::state_currency::StateCurrency;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use ecs_event::impulse;
use system_component_default_gameplay::world_context_3d::WorldContext;
use system_component_default_gameplay::{
    UIEvents,
    traits::{impulse::Impulse, scope::Scope},
};

#[derive(Default)]
#[impulse(GameEvents)]
pub struct ECSSystemGamePointScored {}

impl Scope for ECSSystemGamePointScored {
    fn is_enabled(&mut self, _: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        NetworkModes::all_host()
    }
}
impl Impulse<GameEvents> for ECSSystemGamePointScored {
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
