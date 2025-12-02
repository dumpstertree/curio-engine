use core::{
    collections::{event_queue::EventQueue, game_state::GameState, quaternion::Quaternion, vector2_int::Vector2Int, vector3::Vector3},
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::ecs_event_reciever::{self, InstanceLimiter},
    io::asset_loader::AssetLoader,
    random::Random,
};

use built_in::component::{component_renderer_animated::RendererAnimated, component_renderer_static::Renderer, component_transform::Transform};
use built_in_state::state_network::{self, StateNetwork};
use ecs_event::global_ecs_system_event_reciever;
use hecs::World;
use serde::{Deserialize, Serialize};

use crate::{
    AssetMappingUIDs,
    cards::deck_library::DeckLibrary,
    ecs::components::{component_player::ComponentPlayer, component_view_player::ComponentViewPlayer},
    game_events::GameEvents,
    state::{
        host::state_enounter_mode::StateEncounter,
        state_controller::StateController,
        state_deck::{Deck, StateDeck},
        state_energy::StateEnergy,
        state_position_player::StatePositionEntities,
        state_teams::{self, StateTeamAssignments, Teams},
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
            GameEvents::DidInitializeEncounter(_) => {
                let asset_goblin = AssetLoader::load_model_animated_from_database(AssetMappingUIDs::Goblin.uid());

                let state_teams = game_state.get::<StateTeamAssignments>();
                for team in Teams::all() {
                    if let Some(guids) = state_teams.team_assignments.get(&team) {
                        for guid in guids {
                            println!("init {}", guid);
                            let mut rend = RendererAnimated::default();
                            rend.set_asset(Some(asset_goblin.clone()));
                            // players
                            world.spawn((
                                ComponentViewPlayer::default(),
                                ComponentPlayer::default().set_player_id(*guid),
                                Transform::default()
                                    .set_position(Vector3::new(-5.0, -5.0, 10.0))
                                    .set_rotation(Quaternion::from_euler(Vector3::new(1.0, 0.0, 1.0))),
                                rend,
                            ));
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
