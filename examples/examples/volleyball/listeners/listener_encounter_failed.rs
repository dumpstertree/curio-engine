use crate::game_events::GameEvents;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use ecs_event::impulse;
use system_component_default_gameplay::{
    traits::{impulse::Impulse, scope::Scope},
    context_3d::Context3D,
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
    fn dequeue_event(&mut self, _game_state: &mut GameState, _: &mut Context3D, _event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::EncounterFailed => {
                panic!("encounter failed");
            }
            _ => {}
        }
    }
}
