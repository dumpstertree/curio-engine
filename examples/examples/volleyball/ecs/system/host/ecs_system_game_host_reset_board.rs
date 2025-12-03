use crate::game_board::GameBoard;
use crate::game_events::GameEvents;
use crate::state;
use crate::state::host::state_card_attribute_modifier_stack::StateCardAttributeModifierStack;
use crate::state::host::state_enounter_mode::StateEncounter;
use crate::state::state_ball_mode::{BallModes, StateBallMode};
use crate::state::state_controller::StateController;
use crate::state::state_deck::{Deck, StateDeck};
use crate::state::state_energy::StateEnergy;
use crate::state::state_position_ball::StatePositionBall;
use crate::state::state_position_player::StatePositionEntities;
use crate::state::state_teams::{StateTeamAssignments, Teams};
use core::gameplay::ecs::traits::ecs_event_reciever::{self, InstanceLimiter};
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
};
use ecs_event::global_ecs_system_event_reciever;
use ecs_system::global_ecs_system;
use hecs::World;
use serde::de;
use winit::dpi::Position;

#[global_ecs_system]
#[global_ecs_system_event_reciever(GameEvents)]
pub struct ECSSystemGameResetBoard {}
impl ECSSystemEventless for ECSSystemGameResetBoard {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}
impl InstanceLimiter for ECSSystemGameResetBoard {
    fn is_enabled(&mut self, _: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}

impl ecs_event_reciever::EventReciever<GameEvents> for ECSSystemGameResetBoard {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut World, event_queue: &mut EventQueue, event: &GameEvents) {
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
                                for _ in 0..5 {
                                    y.draw();
                                }
                            }
                        });
                    }
                }
                // reset all blue
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
                                y.0 = GameBoard::get_serving_tile(&Teams::Blue).0;
                                y.1 = GameBoard::get_serving_tile(&Teams::Blue).1;
                            }
                        });
                        // reset shuffle deck
                        game_state.edit::<StateDeck>(|x| {
                            if let Some(y) = x.deck.get_mut(guid) {
                                y.reshuffle();
                                for _ in 0..7 {
                                    y.draw();
                                }
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
