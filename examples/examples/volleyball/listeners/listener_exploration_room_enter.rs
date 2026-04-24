use crate::exploration::exploration_path::RoomTypes;
use crate::game_events::GameEvents;
use crate::listeners::listener_initialize_exploration::{EncounterLibrary, ShopLibrary};
use curio_core::{
    collections::network_modes::NetworkModes,
    collections::{event_queue::EventQueue, ledger::Ledger},
};
use gameplay::context_3d::Context3D;
use gameplay::traits::{impulse::Impulse, scope::Scope};
use impulse::impulse;

#[derive(Default)]
#[impulse(GameEvents)]
pub struct ECsystemGamePointScored {}

impl Scope for ECsystemGamePointScored {
    fn is_enabled(&mut self, _: &mut Ledger) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut Ledger) -> Vec<NetworkModes> {
        NetworkModes::all_host()
    }
}
impl Impulse<GameEvents> for ECsystemGamePointScored {
    fn dequeue_event(&mut self, _ledger: &mut Ledger, _: &mut Context3D, event_queue: &mut EventQueue, event: &GameEvents) {
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
