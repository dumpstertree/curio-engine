use built_in_state::state_network::StateNetwork;
use ecs_event::global_ecs_system_event_reciever;
use ecs_system::global_ecs_system;
use hecs::World;

use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
    extensions::extensions_i32::ExtensionsI32,
    gameplay::ecs::traits::{ecs_event_reciever, ecs_system::ECSSystemEventless},
};

use crate::{
    game_events::GameEvents,
    state::{state_position_ball::StatePositionBall, state_teams::StateTeamAssignments},
};

#[global_ecs_system]
pub struct ECSSystemGameEndTurn {}
impl ECSSystemEventless for ECSSystemGameEndTurn {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
    // fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
    //     vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    // }
}
#[global_ecs_system_event_reciever(GameEvents)]
impl ecs_event_reciever::EventReciever<GameEvents> for ECSSystemGameEndTurn {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut World, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::TurnEnd(id) => {
                // end this turn
                println!("Instance: {}. End Turn {}", game_state.instance_id, id);

                let Some(team) = game_state.get_value2::<StateTeamAssignments>().team_for(id) else {
                    println!("unknown team");
                    return;
                };

                let state_position_ball = game_state.get_value2::<StatePositionBall>();
                let ball_is_on_side = team.on_side(state_position_ball.collun, state_position_ball.row);
                if ball_is_on_side {
                    println!("Point scored for {}!", team.next_team());
                    event_queue.enqueue_event(GameEvents::PointScored(team));
                    return;
                }

                // todo iterate to the next player
                let mut index = -1;
                for x in 0..game_state.all_instance_id.len() as i32 {
                    let other_id = game_state.all_instance_id[x as usize];
                    if *id == other_id {
                        index = x;
                    }
                }

                if index == -1 {
                    println!("Couldnt find curreent player index!");
                    return;
                }
                let state_network = game_state.get_value2::<StateNetwork>();
                let peer_ids = state_network.peer_instance_ids();
                let wrapped_index = (index + 1).repeat(0, peer_ids.len() as i32);
                let new_id = peer_ids[wrapped_index as usize];
                // begin the next player
                event_queue.enqueue_event(GameEvents::TurnBegin(new_id));
            }
            _ => {}
        }
    }
}
