use crate::cards::card_attributes_targets::attribute_target_type_tiles::AttributeTargetTypesTiles;
use crate::cards::card_dependencies::data_dep_empty::DataDepsEmpty;
use crate::ecs::components::component_gameboard_tile::ComponentGameBoardTile;
use crate::exploration::exploration_path::RoomTypes;
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

use curio_core::built_in::record::sys_record_time::SysRecordTime;
use curio_core::{Vector2Int, Vector3};
use curio_core::{
    collections::network_modes::NetworkModes,
    collections::{event_queue::EventQueue, ledger::Ledger},
};
use gameplay::built_in::facet::renderer::renderer_static::RendererStatic;
use gameplay::built_in::facet::renderer_common::RendererCommon;
use gameplay::built_in::facet::transform::transform3d::Transform3D;
use gameplay::context_3d::Context3D;
use gameplay::traits_internal::world_context_common::ContextCommon;

use gameplay::traits::habit::Habit;
use gameplay::traits::scope::Scope;
use habit::habit;

#[habit]
pub struct Instance {}
impl Scope for Instance {
    fn is_enabled(&mut self, ledger: &mut Ledger) -> bool {
        let state_exploration = ledger.read::<StateExploration>();
        state_exploration.exploration.get_cur_room().room_type == RoomTypes::Combat && !state_exploration.is_selecting_next
    }
    fn run_on_instance(&mut self, _ledger: &mut Ledger) -> Vec<NetworkModes> {
        NetworkModes::all_peer()
    }
}
impl Habit for Instance {
    fn tick(&mut self, ledger: &mut Ledger, world: &mut Context3D, _events: &mut EventQueue) {
        let state_mode = ledger.read::<StatePeerInputMode>();
        let state_deck = ledger.read::<StateDeck>();
        let state_index = ledger.read::<StatePeerSelectedCards>();
        if state_mode.mode != InputModes::Manuever && ledger.read::<StatePeerSelectTargets>().enabled.is_none() {
            world.edit::<(&mut Transform3D, &ComponentGameBoardTile, &mut RendererStatic)>(|query| {
                for (_, (_transform, _gameboard_tile, renderer)) in query {
                    renderer.set_enabled(false);
                }
            });
            return;
        }
        world.edit::<(&mut Transform3D, &ComponentGameBoardTile, &mut RendererStatic)>(|query| {
            for (_, (transform, gameboard_tile, renderer)) in query {
                let Some(deck) = state_deck.deck.get(&ledger.instance_id) else {
                    return;
                };

                let state_entity_pos = ledger.read::<StatePositionEntities>();

                let hand = deck.get_cards_from_hand(|x| x.get_manuever_type() != CardTypes::Move);
                let selected = &hand.get(state_index.index as usize);
                let Some(selected) = selected else {
                    return;
                };
                let team = ledger
                    .read::<StateTeamAssignments>()
                    .team_for(&ledger.instance_id)
                    .unwrap();
                let pos_entity = state_entity_pos
                    .positions
                    .get(&ledger.instance_id)
                    .unwrap();
                let pos_ball = ledger.read::<StatePositionBall>();
                let events = selected.get_attributes_events(ledger, ledger.instance_id);
                let mut targets = Vec::new();
                let state_attribute_stack = ledger.read::<StateCardAttributeModifierStack>();
                let s0 = state_attribute_stack.get_flat_stack_for_entity(ledger.instance_id);
                // let s1 = state_attribute_stack.get_flat_stack_for_card(selected.instance_id);
                // let stack = s0.;

                for e in events {
                    for d in e.get_data_dependencies_empty() {
                        match d {
                            DataDepsEmpty::Tiles(attribute_target_types_tiles) => match attribute_target_types_tiles {
                                // AttributeTargetTypesTiles::Select => todo!(),
                                AttributeTargetTypesTiles::SelectInRangeLocalToBall(min, max) => {
                                    for x in min.x..(max.x + 1) {
                                        for y in min.y..(max.y + 1) {
                                            let c = team.convert_dir(x, y + s0.range);
                                            targets.push(Vector2Int::new(pos_ball.column + c.0, pos_ball.row + c.1));
                                        }
                                    }
                                }
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
                let sin = f32::sin(ledger.read::<SysRecordTime>().unscaled_time as f32 * 5.0);

                renderer.set_enabled(targets.contains(&gameboard_tile.tile));
                transform.scale = Vector3::one() + Vector3::one() * 0.1 * sin;
            }
        });
    }
}
