use crate::exploration::exploration_path::RoomTypes;
use crate::game_events::GameEvents;
use crate::listeners::listener_initialize_exploration::{EncounterLibrary, ShopLibrary};
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use ecs_event::global_ecs_system_event_reciever;
use system_component_default_gameplay::world_context::WorldContext;
use system_component_default_gameplay::{
    UIEvents,
    traits::{event_reciever::EventReciever, instance_scope::InstanceLimiter},
};

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
    fn dequeue_event(&mut self, _game_state: &mut GameState, _: &mut WorldContext, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::ExplorationRoomEnter(room) => {
                // log
                println!("Enter Exploration Room: {}", room.guid);

                // get the newly assigned state
                match room.room_type {
                    RoomTypes::Combat => {
                        let roll = EncounterLibrary::random();
                        event_queue.enqueue_event(GameEvents::InitializeEncounter(roll.clone()));
                        event_queue.enqueue_event(GameEvents::ExplorationDidRoomEnterCombat(room.clone(), roll.clone()));
                    }
                    RoomTypes::Shop => {
                        let roll = ShopLibrary::random();
                        event_queue.enqueue_event(GameEvents::InitializeShop(roll.clone()));
                        event_queue.enqueue_event(GameEvents::ExplorationDidRoomEnterShop(room.clone(), roll.clone()));
                    }
                    RoomTypes::Boss => todo!(),
                    RoomTypes::Heal => {
                        event_queue.enqueue_event(GameEvents::ExplorationDidRoomEnterHeal(room.clone()));
                    }
                    RoomTypes::Invalid => todo!(),
                }
            }
            _ => {}
        }
    }
}
