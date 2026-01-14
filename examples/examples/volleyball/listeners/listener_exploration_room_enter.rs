use crate::exploration::exploration_path::RoomTypes;
use crate::game_events::GameEvents;
use crate::listeners::listener_initialize_exploration::{EncounterLibrary, ShopLibrary};
use curio_core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use ecs_event::impulse;
use system_component_default_gameplay::context_3d::Context3D;
use system_component_default_gameplay::traits::{impulse::Impulse, scope::Scope};

#[derive(Default)]
#[impulse(GameEvents)]
pub struct ECSSystemGamePointScored {}

impl Scope for ECSSystemGamePointScored {
    fn is_enabled(&mut self, _: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<NetworkModes> {
        NetworkModes::all_host()
    }
}
impl Impulse<GameEvents> for ECSSystemGamePointScored {
    fn dequeue_event(&mut self, _game_state: &mut GameState, _: &mut Context3D, event_queue: &mut EventQueue, event: &GameEvents) {
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
