use crate::ecs::components::component_ball::ComponentBall;
use crate::ecs::components::component_energy_token::ComponentEnergyToken;
use crate::ecs::components::component_player::ComponentPlayer;
use crate::ecs::components::component_view_player::ComponentViewPlayer;
use crate::game_events::GameEvents;
use crate::listeners::listener_ui_set_mode::UITypes;
use crate::state::host::state_exploration::StateExploration;
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
            GameEvents::ExplorationDidPickRoomComplete => {
                // turn off selection value
                game_state.edit::<StateExploration>(|x| {
                    x.is_selecting_next = false;
                });

                // turn off ui
                event_queue.enqueue_event(system_component_default_gameplay::UIEvents::Close(UIViewTypes::PanelExploration));
            }
            _ => {}
        }
    }
}
