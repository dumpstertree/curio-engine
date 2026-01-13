use crate::UIViewTypes;
use crate::game_events::GameEvents;
use crate::state::host::state_exploration::StateExploration;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use ecs_event::impulse;
use system_component_default_gameplay::context_3d::Context3D;
use system_component_default_gameplay::{
    built_in::impulse::ui_events::UIEvents,
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
    fn dequeue_event(&mut self, game_state: &mut GameState, world: &mut Context3D, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::ExplorationDidPickRoomComplete => {
                // turn off selection value
                game_state.edit::<StateExploration>(|x| {
                    x.is_selecting_next = false;
                });

                // turn off ui
                event_queue.enqueue_event(UIEvents::Close(UIViewTypes::PanelExploration));
            }
            _ => {}
        }
    }
}
