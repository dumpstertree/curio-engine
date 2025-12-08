use crate::exploration::exploration_path::RoomTypes;
use crate::game_events::GameEvents;
use crate::state::host::state_enounter_mode::StateEncounter;
use core::gameplay::ecs::traits::ecs_event_reciever::{self, InstanceLimiter};
use core::gameplay::world_context::WorldContext;
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
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut WorldContext, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::ExplorationRoomExit(room) => {
                println!("Exit Exploration Room: {}", room.guid);

                match room.room_type {
                    RoomTypes::Combat => {
                        // get the current encounter we are leaving
                        let state_encounter = game_state.get::<StateEncounter>();

                        //get the current encounter
                        let encounter = state_encounter.encounter;

                        // notify as to leaving combat room
                        event_queue.enqueue_event(GameEvents::ExplorationDidRoomExitCombat(room.clone(), encounter.clone()));
                    }
                    RoomTypes::Shop => todo!(),
                    RoomTypes::Boss => todo!(),
                    RoomTypes::Heal => {
                        //notify as to leaving healing room
                        event_queue.enqueue_event(GameEvents::ExplorationDidRoomExitHeal(room.clone()));
                    }
                    RoomTypes::Invalid => todo!(),
                }
                // log
            }
            _ => {}
        }
    }
}
