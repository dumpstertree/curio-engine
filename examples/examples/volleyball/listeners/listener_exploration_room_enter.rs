use crate::exploration::exploration_path::RoomTypes;
use crate::game_events::GameEvents;
use crate::listeners::listener_initialize_exploration::EncounterLibrary;
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
    fn dequeue_event(&mut self, _game_state: &mut GameState, _: &mut World, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::ExplorationRoomEnter(room) => {
                // log
                println!("Enter Exploration Room: {}", room.guid);

                // get the newly assigned state
                match room.room_type {
                    // start a new encounter
                    RoomTypes::Combat => {
                        let encounter = EncounterLibrary::random();
                        event_queue.enqueue_event(GameEvents::InitializeEncounter(encounter.clone()));
                        event_queue.enqueue_event(GameEvents::ExplorationDidRoomEnterCombat(room.clone(), encounter.clone()));
                    }
                    // start a new shop
                    RoomTypes::Shop => todo!(),
                    // start a new boss
                    RoomTypes::Boss => todo!(),
                }
            }
            _ => {}
        }
    }
}
