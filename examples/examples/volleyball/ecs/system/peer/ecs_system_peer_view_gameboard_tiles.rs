use crate::cards::card_attributes_targets::attribute_target_type_tiles::AttributeTargetTypesTiles;
use crate::cards::card_dependencies::data_dep_empty::DataDepsEmpty;
use crate::ecs::components::component_gameboard_tile::ComponentGameBoardTile;
use crate::ecs::components::component_player::ComponentPlayer;
use crate::ecs::components::component_view_player::ComponentViewPlayer;
use crate::exploration::exploration_path::{Exploration, RoomTypes};
use crate::game_board::GameBoard;
use crate::state::host::state_card_attribute_modifier_stack::StateCardAttributeModifierStack;
use crate::state::host::state_exploration::StateExploration;
use crate::state::peer::state_peer_input_mode::{InputModes, StatePeerInputMode};
use crate::state::peer::state_peer_select_targets::StatePeerSelectTargets;
use crate::state::peer::state_peer_selected_card::StatePeerSelectedCards;
use crate::state::state_deck::{CardTypes, StateDeck};
use crate::state::state_position_ball::StatePositionBall;
use crate::state::state_position_player::StatePositionEntities;
use crate::state::state_teams::StateTeamAssignments;
use crate::{AssetMappingUIDs, game_events};
use built_in::component::component_renderer_animated::RendererAnimated;
use built_in::component::component_renderer_static::Renderer;
use built_in::component::component_renderer_text::RendererCommon;
use built_in_state::state_debug::StateDebug;
use built_in_state::state_network::StateNetwork;
use built_in_state::state_time::TimeState;
use core::collections::game_state;
use core::collections::quaternion::Quaternion;
use core::collections::vector2_int::Vector2Int;
use core::collections::vector3::Vector3;
use core::gameplay::ecs::component::component_transform::Transform;
use core::gameplay::world_context::{WorldContext, WorldContextCommon};
use core::io::asset_loader::AssetLoader;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
};
use ecs_system::global_ecs_system;
use hecs::World;
use mcts::Player;
use winit::dpi::Position;

#[global_ecs_system]
pub struct Instance {}
impl ECSSystemEventless for Instance {
    fn run_on_instance(&mut self, _game_state: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        NetworkModes::all_peer()
    }
    fn is_enabled(&mut self, game_state: &mut GameState, _: &mut WorldContext) -> bool {
        let state_exploration = game_state.get::<StateExploration>();
        state_exploration.exploration.get_cur_room().room_type == RoomTypes::Combat && !state_exploration.is_selecting_next
    }
    fn tick(&mut self, game_state: &mut GameState, world: &mut WorldContext, events: &mut EventQueue) {
        let state_mode = game_state.get::<StatePeerInputMode>();
        let state_deck = game_state.get::<StateDeck>();
        let state_index = game_state.get::<StatePeerSelectedCards>();
        if state_mode.mode != InputModes::Manuever && game_state.get::<StatePeerSelectTargets>().enabled.is_none() {
            world.query_mut::<(&mut Transform, &ComponentGameBoardTile, &mut Renderer)>(|query| {
                for (_, (transform, gameboard_tile, renderer)) in query {
                    renderer.set_enabled(false);
                }
            });
            return;
        }
        world.query_mut::<(&mut Transform, &ComponentGameBoardTile, &mut Renderer)>(|query| {
            for (_, (transform, gameboard_tile, renderer)) in query {
                let Some(deck) = state_deck.deck.get(&game_state.instance_id) else {
                    return;
                };

                let state_entity_pos = game_state.get::<StatePositionEntities>();

                let hand = deck.get_cards_from_hand(|x| x.get_manuever_type() != CardTypes::Move);
                let selected = &hand.get(state_index.index as usize);
                let Some(selected) = selected else {
                    return;
                };
                let team = game_state
                    .get::<StateTeamAssignments>()
                    .team_for(&game_state.instance_id)
                    .unwrap();
                let pos_entity = state_entity_pos
                    .positions
                    .get(&game_state.instance_id)
                    .unwrap();
                let pos_ball = game_state.get::<StatePositionBall>();
                let events = selected.get_attributes_events(game_state, game_state.instance_id);
                let mut targets = Vec::new();
                let state_attribute_stack = game_state.get::<StateCardAttributeModifierStack>();
                let s0 = state_attribute_stack.get_flat_stack_for_entity(game_state.instance_id);
                // let s1 = state_attribute_stack.get_flat_stack_for_card(selected.instance_id);
                // let stack = s0.;

                for e in events {
                    for d in e.get_data_dependencies_empty() {
                        match d {
                            DataDepsEmpty::Tiles(attribute_target_types_tiles) => match attribute_target_types_tiles {
                                // AttributeTargetTypesTiles::Select => todo!(),
                                AttributeTargetTypesTiles::RandomInRangeGlobal(min, max) => {
                                    for x in min.x..(max.x + 1) {
                                        for y in min.y..(max.y + 1) {
                                            targets.push(Vector2Int::new(x, y));
                                        }
                                    }
                                }
                                AttributeTargetTypesTiles::RandomInRangeLocalToBall(min, max) => {
                                    for x in min.x..(max.x + 1) {
                                        for y in min.y..(max.y + 1) {
                                            let c = team.convert_dir(x, y + s0.range);
                                            targets.push(Vector2Int::new(pos_ball.column + c.0, pos_ball.row + c.1));
                                        }
                                    }
                                }
                                AttributeTargetTypesTiles::RandomInRangeLocalToUser(min, max) => {
                                    for x in min.x..(max.x + 1) {
                                        for y in min.y..(max.y + 1) {
                                            let c = team.convert_dir(x, y + s0.range);
                                            targets.push(Vector2Int::new(pos_entity.0 + c.0, pos_entity.1 + c.1));
                                        }
                                    }
                                }
                                AttributeTargetTypesTiles::RandomOnTeamUser | AttributeTargetTypesTiles::SelectOnTeamUser => {
                                    let min = GameBoard::get_bounds_min_for_team(&team);
                                    let max = GameBoard::get_bounds_max_for_team(&team);
                                    for x in min.x..(max.x + 1) {
                                        for y in min.y..(max.y + 1) {
                                            targets.push(Vector2Int::new(x, y));
                                        }
                                    }
                                }
                                AttributeTargetTypesTiles::RandomOnTeamOpponent | AttributeTargetTypesTiles::SelectOnTeamOpponent => {
                                    let min = GameBoard::get_bounds_min_for_team(&team.next_team());
                                    let max = GameBoard::get_bounds_max_for_team(&team.next_team());
                                    for x in min.x..(max.x + 1) {
                                        for y in min.y..(max.y + 1) {
                                            targets.push(Vector2Int::new(x, y));
                                        }
                                    }
                                }
                                AttributeTargetTypesTiles::RandomAny | AttributeTargetTypesTiles::SelectAny => {
                                    let min = GameBoard::get_bounds_min();
                                    let max = GameBoard::get_bounds_max();
                                    for x in min.x..(max.x + 1) {
                                        for y in min.y..(max.y + 1) {
                                            targets.push(Vector2Int::new(x, y));
                                        }
                                    }
                                }
                                _ => {}
                            },
                            _ => {}
                        }
                    }
                }
                let sin = f32::sin(game_state.get::<TimeState>().unscaled_time as f32 * 5.0);

                renderer.set_enabled(targets.contains(&gameboard_tile.tile));
                transform.scale = Vector3::one() + Vector3::one() * 0.1 * sin;
            }
        });
    }
}
