use crate::game_board::GameBoard;
use crate::game_events::GameEvents;
use crate::state::host::state_card_attribute_modifier_stack::StateCardAttributeModifierStack;
use crate::state::state_ball_mode::{BallModes, StateBallMode};
use crate::state::state_deck::StateDeck;
use crate::state::state_energy::StateEnergy;
use crate::state::state_position_ball::StatePositionBall;
use crate::state::state_position_player::StatePositionEntities;
use crate::state::state_teams::{StateTeamAssignments, Teams};
use curio_core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use gameplay::context_3d::Context3D;
use gameplay::traits::{impulse::Impulse, scope::Scope};
use impulse::impulse;

#[derive(Default)]
#[impulse(GameEvents)]
pub struct ECSSystemGameResetBoard {}
impl Scope for ECSSystemGameResetBoard {
    fn is_enabled(&mut self, _: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}

impl Impulse<GameEvents> for ECSSystemGameResetBoard {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut Context3D, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::ResetBoard(serving_team) => {
                println!("Board Reset------------------------------------------------");
                // setup ball mode
                game_state.edit::<StateBallMode>(|x| x.mode = BallModes::Serve);

                // set ball position
                game_state.edit::<StatePositionBall>(|x| {
                    x.column = GameBoard::get_serving_tile(&serving_team).0;
                    x.row = GameBoard::get_serving_tile(&serving_team).1;
                });

                // reset attributes
                game_state.edit::<StateCardAttributeModifierStack>(|x| {
                    x.clear_all();
                });

                // reset all red
                let state_team = game_state.get::<StateTeamAssignments>();
                if let Some(guids) = state_team.team_assignments.get(&Teams::Red) {
                    for guid in guids {
                        // reset energy
                        game_state.edit::<StateEnergy>(|x| {
                            if let Some(y) = x.all_players.get_mut(guid) {
                                y.0 = y.1;
                            }
                        });
                        // reset position
                        game_state.edit::<StatePositionEntities>(|x| {
                            if let Some(y) = x.positions.get_mut(guid) {
                                y.0 = GameBoard::get_serving_tile(&Teams::Red).0;
                                y.1 = GameBoard::get_serving_tile(&Teams::Red).1;
                            }
                        });
                        // reset shuffle deck
                        game_state.edit::<StateDeck>(|x| {
                            if let Some(y) = x.deck.get_mut(guid) {
                                y.reshuffle();
                                y.draw(5);
                            }
                        });
                    }
                }
                // reset all blue
                let mut pos_offset = 0;
                if let Some(guids) = state_team.team_assignments.get(&Teams::Blue) {
                    for guid in guids {
                        // reset energy
                        game_state.edit::<StateEnergy>(|x| {
                            if let Some(y) = x.all_players.get_mut(guid) {
                                y.0 = y.1;
                            }
                        });
                        // reset position
                        game_state.edit::<StatePositionEntities>(|x| {
                            if let Some(y) = x.positions.get_mut(guid) {
                                y.0 = GameBoard::get_serving_tile(&Teams::Blue).0 - pos_offset;
                                y.1 = GameBoard::get_serving_tile(&Teams::Blue).1 - pos_offset;
                            }
                        });
                        pos_offset += 1;
                        // reset shuffle deck
                        game_state.edit::<StateDeck>(|x| {
                            if let Some(y) = x.deck.get_mut(guid) {
                                y.reshuffle();
                                y.draw(5);
                            }
                        });
                    }
                }

                event_queue.enqueue_event(GameEvents::TurnBegin(*serving_team));
            }
            _ => {}
        }
    }
}
