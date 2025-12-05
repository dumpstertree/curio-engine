use built_in::component::{
    component_renderer_animated::RendererAnimated,
    component_renderer_static::Renderer,
    component_renderer_text::{ComponentRendererText, RendererCommon},
    component_transform::Transform,
};
use built_in_state::state_camera::CameraState;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState, quaternion::Quaternion, vector2::Vector2, vector3::Vector3},
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::ecs_event_reciever::{self, InstanceLimiter},
    io::asset_loader::AssetLoader,
};
use ecs_event::global_ecs_system_event_reciever;
use hecs::World;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    AssetMappingUIDs,
    cards::card_instance::CardInstance,
    ecs::components::{component_card::ComponentCard, component_energy_token::ComponentEnergyToken, component_player::ComponentPlayer, component_ui_ball_state::ComponentUIBallState, component_ui_score::ComponentUIScoreState, component_ui_turn::ComponentUITurnState},
    game_events::GameEvents,
    state::{
        peer::state_peer_entity_ids::{EntityIDTypes, StateEntityIDs},
        state_deck::{Deck, StateDeck},
        state_teams::{StateTeamAssignments, Teams},
    },
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
        NetworkModes::all_peer()
    }
}
// Impl - Listener
impl ecs_event_reciever::EventReciever<GameEvents> for Listener {
    fn dequeue_event(&mut self, game_state: &mut GameState, world: &mut World, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::DisableUIHealing => {
                let id = EntityIDTypes::UIPanelHealing;
                for e in game_state.get::<StateEntityIDs>().get(id.clone()) {
                    let _ = world.despawn(e);
                }
                game_state.edit::<StateEntityIDs>(|x| x.clear(id.clone()));
            }
            _ => {}
        }
    }
}
impl Listener {}
