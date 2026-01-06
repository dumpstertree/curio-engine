use crate::game_events::GameEvents;
use crate::state::host::state_exploration::StateExploration;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use ecs_event::global_ecs_system_event_reciever;
use system_component_default_gameplay::{
    traits::{impulse::Impulse, scope::Scope},
    world_context::WorldContext,
};

#[derive(Default)]
#[global_ecs_system_event_reciever(GameEvents)]
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
            GameEvents::ExplorationPickRoomComplete(room) => {
                // // edit the encounter state to move to the next room
                game_state.edit::<StateExploration>(|x| {
                    // progress the exploration
                    x.exploration.next(&room.guid);
                });

                // enter current room
                let state_exploration = game_state.get::<StateExploration>();
                event_queue.enqueue_event(GameEvents::ExplorationRoomEnter(state_exploration.exploration.get_cur_room()));

                // close the ui
                event_queue.enqueue_event(GameEvents::ExplorationDidPickRoomComplete);
            }
            _ => {}
        }
    }
}
