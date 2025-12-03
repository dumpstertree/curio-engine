use crate::game_events::GameEvents;
use crate::state::host::state_exploration::StateExploration;
use core::collections::event_queue;
use core::gameplay::ecs::traits::ecs_event_reciever::{self, InstanceLimiter};
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use ecs_event::global_ecs_system_event_reciever;
use hecs::World;

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
impl ecs_event_reciever::EventReciever<GameEvents> for ECSSystemGamePointScored {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut World, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::EncounterPassed => {
                // log
                println!("Encounter Passed");

                // exit current room
                let state_exploration = game_state.get::<StateExploration>();
                event_queue.enqueue_event(GameEvents::ExplorationRoomExit(state_exploration.exploration.get_cur_room()));

                // edit the encounter state to move to the next room
                game_state.edit::<StateExploration>(|x| {
                    let next_rooms = x.exploration.get_next_room();
                    let selected_next_room = &next_rooms[0];

                    // progress the exploration
                    x.exploration.next(&selected_next_room.guid);
                });

                // enter current room
                let state_exploration = game_state.get::<StateExploration>();
                event_queue.enqueue_event(GameEvents::ExplorationRoomEnter(state_exploration.exploration.get_cur_room()));
            }
            _ => {}
        }
    }
}
