use crate::{
    game_events::GameEvents,
    state::{
        state_ball_mode::{BallModes, StateBallMode},
        state_deck::CardTypes,
        state_energy::StateEnergy,
        state_position_ball::StatePositionBall,
        state_teams::StateTeamAssignments,
    },
};
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::{ecs_event_reciever, ecs_system::ECSSystemEventless},
};
use ecs_event::global_ecs_system_event_reciever;
use ecs_system::global_ecs_system;
use hecs::World;

#[global_ecs_system]
pub struct ECSSystemGameRequestManuever {}
impl ECSSystemGameRequestManuever {}
impl ECSSystemEventless for ECSSystemGameRequestManuever {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}
#[global_ecs_system_event_reciever(GameEvents)]
impl ecs_event_reciever::EventReciever<GameEvents> for ECSSystemGameRequestManuever {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut World, _: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::PlayCard(id, card) => {
                println!("Played Card ({})", card.title);
                match card.card_type {
                    CardTypes::Set => {
                        game_state.edit::<StateBallMode>(|x| {
                            x.mode = BallModes::Set;
                        });
                    }
                    CardTypes::Bump => {
                        let team = &game_state
                            .get_value2::<StateTeamAssignments>()
                            .team_for(id)
                            .unwrap();
                        game_state.edit::<StateBallMode>(|x| {
                            x.mode = BallModes::Bump;
                        });
                        game_state.edit::<StatePositionBall>(|x| {
                            // convert based on team
                            let diff = team.convert_dir(0, 1);
                            // move
                            x.collun = x.collun + diff.0;
                            x.row = x.row + diff.1;

                            println!("Ball moved for team ({}): ({},{}) -> ({},{})", team, x.collun - diff.0, x.row - diff.1, x.collun, x.row);
                        });
                    }
                    CardTypes::Spike => {
                        let team = &game_state
                            .get_value2::<StateTeamAssignments>()
                            .team_for(id)
                            .unwrap();

                        game_state.edit::<StateBallMode>(|x| {
                            x.mode = BallModes::Spike;
                        });
                        game_state.edit::<StatePositionBall>(|x| {
                            // convert based on team
                            let diff = team.convert_dir(0, 2);
                            // move
                            x.collun = x.collun + diff.0;
                            x.row = x.row + diff.1;

                            println!("Ball moved for team ({}): ({},{}) -> ({},{})", team, x.collun - diff.0, x.row - diff.1, x.collun, x.row);
                        });
                    }
                    CardTypes::Rest => {
                        game_state.edit::<StateEnergy>(|x| {
                            let Some(energy) = x.all_players.get_mut(id) else {
                                return;
                            };

                            energy.1 = energy.1 - 1;
                            energy.0 = energy.1;
                        });
                    }
                    _ => {
                        println!("Unsupported Card Type ")
                    }
                }
            }
            _ => {}
        }
    }
}
