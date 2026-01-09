use crate::UIViewTypes;
use crate::game_events::GameEvents;
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
pub struct Listener {}

impl Scope for Listener {
    fn is_enabled(&mut self, _: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        NetworkModes::all_peer()
    }
}
impl Impulse<GameEvents> for Listener {
    fn dequeue_event(&mut self, game_state: &mut GameState, world: &mut WorldContext, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::ExplorationDidRoomExitShop(_, _) => {
                println!("exit shop room");
                // change ui
                // event_queue.enqueue_event(GameEvents::SetUIMode(UITypes::None));
                event_queue.enqueue_event(system_component_default_gameplay::UIEvents::Close(UIViewTypes::PanelShop));
            }
            _ => {}
        }
    }
}
