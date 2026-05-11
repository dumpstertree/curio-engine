use crate::exploration::exploration_path::RoomTypes;
use crate::game_events::GameEvents;
use crate::state::host::state_enounter_mode::StateEncounter;
use crate::state::host::state_shop::StateShop;
use curio_core::{
    collections::{event_queue::Nerve, ledger::Ledger},
    network_modes::NetworkModes,
};
use gameplay::{
    context_3d::Context3D,
    traits::{impulse::Impulse, scope::Scope},
};
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
    fn dequeue_event(&mut self, ledger: &mut Ledger, _: &mut Context3D, event_queue: &mut Nerve, event: &GameEvents) {
        match event {
            GameEvents::ExplorationRoomExit(room) => {
                println!("Exit Exploration Room: {}", room.guid);

                match room.room_type {
                    RoomTypes::Combat => {
                        // get the current encounter we are leaving
                        let state_encounter = ledger.read::<StateEncounter>();

                        //get the current encounter
                        let encounter = &state_encounter.encounter;

                        // notify as to leaving combat room
                        event_queue.enqueue_event(GameEvents::ExplorationDidRoomExitCombat(room.clone(), encounter.clone()));
                        event_queue.enqueue_event(GameEvents::FinalizeEncounter(encounter.clone()));
                    }
                    RoomTypes::Shop => {
                        // get the current encounter we are leaving
                        let state_shop = ledger.read::<StateShop>();

                        //get the current encounter
                        let shop = &state_shop.shop;

                        // notify as to leaving combat room
                        event_queue.enqueue_event(GameEvents::ExplorationDidRoomExitShop(room.clone(), shop.clone()));
                        event_queue.enqueue_event(GameEvents::FinalizeShop(shop.clone()));
                    }
                    RoomTypes::Heal => {
                        //notify as to leaving healing room
                        event_queue.enqueue_event(GameEvents::ExplorationDidRoomExitHeal(room.clone()));
                    }
                    RoomTypes::Boss => todo!(),
                    RoomTypes::Invalid => todo!(),
                }
                // log
            }
            _ => {}
        }
    }
}
