use crate::ecs::components::component_ball::ComponentBall;
use crate::ecs::components::component_gameboard_selection::ComponentGameBoardSelection;
use crate::ecs::components::component_gameboard_tile::ComponentGameBoardTile;
use crate::ecs::components::component_player::ComponentPlayer;
use crate::ecs::components::component_view_player::ComponentViewPlayer;
use crate::game_board::GameBoard;
use crate::game_events::GameEvents;
use crate::listeners::listener_ui_set_mode::UITypes;
use crate::state::host::state_entity_visual::StateVisualEntity;
use crate::state::peer::state_peer_entity_ids::{EntityIDTypes, StateEntityIDs};
use crate::state::state_teams::{StateTeamAssignments, Teams};
use crate::{AssetMappingUIDs, UIViewTypes};

use core::collections::quaternion::Quaternion;
use core::collections::vector2_int::Vector2Int;
use core::collections::vector3::Vector3;

use core::io::asset_loader::AssetLoader;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use ecs_event::impulse;
use system_component_default_gameplay::built_in::facet::renderer::renderer_dynamic::RendererDynamic;
use system_component_default_gameplay::built_in::facet::renderer::renderer_static::RendererStatic;
use system_component_default_gameplay::built_in::facet::transform::transform3d::Transform3D;
use system_component_default_gameplay::built_in::impulse::ui_events::UIEvents;
use system_component_default_gameplay::context_3d::Context3D;
use system_component_default_gameplay::traits::{impulse::Impulse, scope::Scope};

#[derive(Default)]
#[impulse(GameEvents)]
pub struct Listener {}

impl Scope for Listener {
    fn is_enabled(&mut self, _: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        NetworkModes::all_peer()
    }
}
impl Impulse<GameEvents> for Listener {
    fn dequeue_event(&mut self, game_state: &mut GameState, world: &mut Context3D, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::ExplorationDidRoomEnterCombat(_, _) => {
                println!("enter combat room");
                // add all entities to world
                Self::spawn_entities(game_state, world);
                // add background to world
                Self::spawn_background(game_state, world);
                // add score to world
                Self::spawn_ball(game_state, world);
                // add the tile visuals
                Self::spawn_tiles(game_state, world);
                // spawn tile selection
                Self::spawn_tile_select(game_state, world);

                // change ui
                event_queue.enqueue_event(GameEvents::SetUIMode(UITypes::Encounter));
                event_queue.enqueue_event(UIEvents::Open(UIViewTypes::HudEncounterBallMode));
                event_queue.enqueue_event(UIEvents::Open(UIViewTypes::HudEncounterEnergy));
                event_queue.enqueue_event(UIEvents::Open(UIViewTypes::HudEncounterScore));
                event_queue.enqueue_event(UIEvents::Open(UIViewTypes::HudEncounterTurn));
                event_queue.enqueue_event(UIEvents::Open(UIViewTypes::HudPreviouslyPlayed));
                event_queue.enqueue_event(UIEvents::Open(UIViewTypes::HUDHeat));
                event_queue.enqueue_event(UIEvents::Open(UIViewTypes::HudEncounterCards));
            }
            _ => {}
        }
    }
}
impl Listener {
    pub fn spawn_tile_select(game_state: &mut GameState, world: &mut Context3D) {
        let asset = Some(AssetLoader::load_model_static_from_database(AssetMappingUIDs::Ball.uid()));
        world
            .spawn("w", Transform3D::default().set_position(Vector3::up() * 0.05))
            .add_facet(ComponentGameBoardSelection::default())
            .add_facet(RendererStatic::default().set_asset(asset.clone()));
    }
    pub fn spawn_tiles(game_state: &mut GameState, world: &mut Context3D) {
        let asset = Some(AssetLoader::load_model_static_from_database(AssetMappingUIDs::GameBoardTileActive.uid()));
        for team in Teams::all() {
            let min = GameBoard::get_bounds_min_for_team(&team);
            let max = GameBoard::get_bounds_max_for_team(&team);

            for x in min.x..(max.x + 1) {
                for z in min.y..(max.y + 1) {
                    let pos = GameBoard::get_world_position(x, z);
                    world
                        .spawn("w", Transform3D::default().set_position(pos + Vector3::up() * 0.05))
                        .add_facet(ComponentGameBoardTile::default().set_tile(Vector2Int::new(x, z)))
                        .add_facet(RendererStatic::default().set_asset(asset.clone()));
                }
            }
        }
    }
    pub fn spawn_ball(game_state: &mut GameState, world: &mut Context3D) {
        let mut r = RendererStatic::default();
        r = r.set_asset(Some(AssetLoader::load_model_static_from_database(AssetMappingUIDs::Ball.uid())));

        let e = world
            .spawn("", Transform3D::default().set_scale(Vector3::one() * 0.5))
            .add_facet(ComponentBall::default())
            .add_facet(r);

        game_state.edit::<StateEntityIDs>(|x| {
            x.add(EntityIDTypes::Ball, e.clone());
        });
    }

    pub fn spawn_background(game_state: &mut GameState, world: &mut Context3D) {
        // let spine = AssetLoader::load_spine_from_path("path");
        let asset_court = AssetLoader::load_model_static_from_database(AssetMappingUIDs::Court.uid());

        let e = world
            .spawn("", Transform3D::default().set_rotation(Quaternion::from_euler(Vector3::new(0.0, 90.0, 0.0))))
            .add_facet(RendererStatic::default().set_asset(Some(asset_court)));

        game_state.edit::<StateEntityIDs>(|x| {
            x.add(EntityIDTypes::Background, e.clone());
        });
    }

    pub fn spawn_entities(game_state: &mut GameState, world: &mut Context3D) {
        let state_entity_visual = game_state.get::<StateVisualEntity>();
        let state_teams = game_state.get::<StateTeamAssignments>();
        for team in Teams::all() {
            if let Some(guids) = state_teams.team_assignments.get(&team) {
                for guid in guids {
                    let asset_id = state_entity_visual
                        .all
                        .get(guid)
                        .unwrap_or(&AssetMappingUIDs::Goblin);

                    let asset_goblin = AssetLoader::load_model_animated_from_database(asset_id.uid());
                    let mut rend = RendererDynamic::default();
                    rend.set_asset(Some(asset_goblin.clone()));
                    // players
                    let e = world
                        .spawn(
                            "",
                            Transform3D::default()
                                .set_position(Vector3::new(-5.0, -5.0, 10.0))
                                .set_rotation(Quaternion::from_euler(Vector3::new(1.0, 0.0, 1.0))),
                        )
                        .add_facet(ComponentViewPlayer::default())
                        .add_facet(ComponentPlayer::default().set_player_id(*guid))
                        .add_facet(rend);
                    game_state.edit::<StateEntityIDs>(|x| {
                        x.add(EntityIDTypes::Entities, e.clone());
                    });
                }
            }
        }
    }
}
