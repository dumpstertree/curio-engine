use crate::game_board::GameBoard;
use crate::game_events::GameEvents;
use crate::state::host::state_card_attribute_modifier_stack::StateCardAttributeModifierStack;
use crate::state::state_ball_mode::{BallModes, StateBallMode};
use crate::state::state_deck::StateDeck;
use crate::state::state_energy::StateEnergy;
use crate::state::state_position_ball::StatePositionBall;
use crate::state::state_position_player::StatePositionPlayer;
use crate::state::state_teams::StateTeamAssignments;
use core::gameplay::ecs::traits::ecs_event_reciever;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
};
use ecs_event::global_ecs_system_event_reciever;
use ecs_system::global_ecs_system;
use hecs::World;

#[global_ecs_system]
pub struct ECSSystemGameResetBoard {}
impl ECSSystemEventless for ECSSystemGameResetBoard {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}

#[global_ecs_system_event_reciever(GameEvents)]
impl ecs_event_reciever::EventReciever<GameEvents> for ECSSystemGameResetBoard {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut World, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::ResetBoard(team) => {
                println!("Board Reset");
                // setup ball mode
                game_state.edit::<StateBallMode>(|x| x.mode = BallModes::Serve);

                // position -> player
                let state_team_assignments = game_state.get_value2::<StateTeamAssignments>();
                game_state.edit::<StatePositionPlayer>(|x| {
                    for y in x.positions.iter_mut() {
                        let Some(team) = state_team_assignments.team_for(y.0) else {
                            continue;
                        };

                        let serving_tile = GameBoard::get_serving_tile(&team);
                        y.1.0 = serving_tile.0;
                        y.1.1 = serving_tile.1;
                    }
                });

                // deck
                game_state.edit::<StateDeck>(|x| {
                    for y in x.deck.iter_mut() {
                        y.1.reshuffle();
                        for _ in 0..5 {
                            y.1.draw();
                        }
                    }
                });
                // energy
                game_state.edit::<StateEnergy>(|x| {
                    for y in x.all_players.iter_mut() {
                        y.1.0 = 5;
                        y.1.1 = 5;
                    }
                });

                let state_teams = game_state.get_value2::<StateTeamAssignments>();
                let state_position_player = game_state.get_value2::<StatePositionPlayer>();
                //
                let Some(team_members) = state_teams.team_assignments.get(team) else {
                    return;
                };

                let Some(first_member) = team_members.get(0) else {
                    return;
                };

                let Some(first_member_pos) = state_position_player.positions.get(first_member) else {
                    return;
                };

                // position -> ball
                game_state.edit::<StatePositionBall>(|x| {
                    x.column = first_member_pos.0;
                    x.row = first_member_pos.1;
                });
                game_state.edit::<StateCardAttributeModifierStack>(|x| {
                    x.clear_all();
                });

                // start the game
                event_queue.enqueue_event(GameEvents::TurnBegin(*first_member));
            }
            _ => {}
        }
    }
}
