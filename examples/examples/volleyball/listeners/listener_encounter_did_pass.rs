use crate::UIViewTypes;
use crate::game_events::GameEvents;
use curio_core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use ecs_event::impulse;
use gameplay::context_3d::Context3D;
use gameplay::{
    built_in::impulse::ui_events::UIEvents,
    traits::{impulse::Impulse, scope::Scope},
};

#[derive(Default)]
#[impulse(GameEvents)]
pub struct ECSSystemGamePointScored {}

impl Scope for ECSSystemGamePointScored {
    fn is_enabled(&mut self, _: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<NetworkModes> {
        NetworkModes::all_peer()
    }
}
impl Impulse<GameEvents> for ECSSystemGamePointScored {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut Context3D, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::DidEncounterPass => {
                // request leave room
                event_queue.enqueue_event(UIEvents::Open(UIViewTypes::PanelRewards));
            }
            _ => {}
        }
    }
}
