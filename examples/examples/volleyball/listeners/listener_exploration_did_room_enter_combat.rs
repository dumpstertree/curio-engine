use crate::AssetMappingUIDs;
use crate::ecs::components::component_ball::ComponentBall;
use crate::ecs::components::component_energy_token::ComponentEnergyToken;
use crate::ecs::components::component_player::ComponentPlayer;
use crate::ecs::components::component_view_player::ComponentViewPlayer;
use crate::game_events::GameEvents;
use crate::listeners::listener_ui_set_mode::UITypes;
use crate::state::peer::state_peer_entity_ids::{EntityIDTypes, StateEntityIDs};
use crate::state::state_teams::{StateTeamAssignments, Teams};
use built_in::component::component_renderer_animated::RendererAnimated;
use built_in::component::component_renderer_static::Renderer;
use built_in::component::component_transform::Transform;
use core::collections::quaternion::Quaternion;
use core::collections::vector3::Vector3;
use core::gameplay::ecs::traits::ecs_event_reciever::{self, InstanceLimiter};
use core::io::asset_loader::AssetLoader;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use ecs_event::global_ecs_system_event_reciever;
use hecs::World;

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
impl ecs_event_reciever::EventReciever<GameEvents> for Listener {
    fn dequeue_event(&mut self, game_state: &mut GameState, world: &mut World, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::ExplorationDidRoomEnterCombat(_, _) => {
                println!("enter combat room");
                // add all entities to world
                Self::spawn_entities(game_state, world);
                // add background to world
                Self::spawn_background(game_state, world);
                // add score to world
                Self::spawn_ball(game_state, world);

                // change ui
                event_queue.enqueue_event(GameEvents::SetUIMode(UITypes::Encounter));
            }
            _ => {}
        }
    }
}
impl Listener {
    pub fn spawn_ball(game_state: &mut GameState, world: &mut World) {
        let mut r = Renderer::default();
        r = r.set_asset(Some(AssetLoader::load_model_static_from_database(AssetMappingUIDs::Ball.uid())));
        let e = world.spawn((Transform::default(), r, ComponentBall::default()));
        game_state.edit::<StateEntityIDs>(|x| {
            x.add(EntityIDTypes::Ball, e);
        });
    }

    pub fn spawn_background(game_state: &mut GameState, world: &mut World) {
        // let spine = AssetLoader::load_spine_from_path("path");
        let asset_court = AssetLoader::load_model_static_from_database(AssetMappingUIDs::Court.uid());

        // court
        let e = world.spawn((
            Transform::default()
                .set_position(Vector3::new(0.0, 0.0, 0.0))
                .set_rotation(Quaternion::from_euler(Vector3::new(0.0, -90.0, 0.0))),
            Renderer::default().set_asset(Some(asset_court)),
        ));

        game_state.edit::<StateEntityIDs>(|x| {
            x.add(EntityIDTypes::Background, e);
        });
    }

    pub fn spawn_entities(game_state: &mut GameState, world: &mut World) {
        let asset_goblin = AssetLoader::load_model_animated_from_database(AssetMappingUIDs::Goblin.uid());

        let state_teams = game_state.get::<StateTeamAssignments>();
        for team in Teams::all() {
            if let Some(guids) = state_teams.team_assignments.get(&team) {
                for guid in guids {
                    let mut rend = RendererAnimated::default();
                    rend.set_asset(Some(asset_goblin.clone()));
                    // players
                    let e = world.spawn((
                        ComponentViewPlayer::default(),
                        ComponentPlayer::default().set_player_id(*guid),
                        Transform::default()
                            .set_position(Vector3::new(-5.0, -5.0, 10.0))
                            .set_rotation(Quaternion::from_euler(Vector3::new(1.0, 0.0, 1.0))),
                        rend,
                    ));
                    game_state.edit::<StateEntityIDs>(|x| {
                        x.add(EntityIDTypes::Entities, e);
                    });
                }
            }
        }
    }
}
