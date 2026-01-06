use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};

use ecs_event::global_ecs_system_event_reciever;
use system_component_default_gameplay::{ecs_event_reciever::{EventReciever, InstanceLimiter}, world_context::WorldContext};

use crate::{
    game_events::GameEvents,
    state::host::state_shop::{Shop, StateShop},
};

#[derive(Default)]
#[global_ecs_system_event_reciever(GameEvents)]
pub struct Listener {}

// Impl - Instance
impl InstanceLimiter for Listener {
    fn is_enabled(&mut self, _game_state: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _game_state: &mut GameState) -> Vec<NetworkModes> {
        NetworkModes::all_host()
    }
}
// Impl - Listener
impl EventReciever<GameEvents> for Listener {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut WorldContext, _: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::FinalizeShop(_) => {
                // clear shop
                game_state.edit::<StateShop>(|x| {
                    x.shop = Shop::new(vec![]);
                });
            }
            _ => {}
        }
    }
}
