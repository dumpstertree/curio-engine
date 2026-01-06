use crate::game_events::GameEvents;
use crate::state::peer::state_peer_entity_ids::{EntityIDTypes, StateEntityIDs};
use crate::state::state_teams::{StateTeamAssignments, Teams};
use crate::{AssetMappingUIDs, UIViewTypes};
use core::io::asset_loader::AssetLoader;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use ecs_event::global_ecs_system_event_reciever;
use hecs::World;
use system_component_default_gameplay::ecs_event_reciever::{EventReciever, InstanceLimiter};
use system_component_default_gameplay::world_context::WorldContext;

#[derive(Default)]
#[global_ecs_system_event_reciever(GameEvents)]
pub struct Listener {}

impl InstanceLimiter for Listener {
    fn is_enabled(&mut self, _: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        NetworkModes::all_peer()
    }
}
impl EventReciever<GameEvents> for Listener {
    fn dequeue_event(&mut self, game_state: &mut GameState, world: &mut WorldContext, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::ExplorationDidRoomEnterShop(_, _) => {
                println!("enter Shop room");

                // open ui
                event_queue.enqueue_event(system_component_default_gameplay::UIEvents::Open(UIViewTypes::PanelShop));
            }
            _ => {}
        }
    }
}
