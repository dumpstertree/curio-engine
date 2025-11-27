use crate::{game_events::GameEvents, state::state_position_player::StatePositionPlayer};
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::{
        ecs_event_reciever::{self, InstanceLimiter},
        ecs_system::ECSSystemEventless,
    },
};
use ecs_event::global_ecs_system_event_reciever;
use ecs_system::global_ecs_system;
use hecs::World;

#[global_ecs_system]
#[global_ecs_system_event_reciever(GameEvents)]
pub struct EventReciever {}
impl ECSSystemEventless for EventReciever {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}
impl InstanceLimiter for EventReciever {
    fn is_enabled(&mut self, _: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}
impl ecs_event_reciever::EventReciever<GameEvents> for EventReciever {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut World, _: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::ApplyCardAttributeEventMoveEntity(player_ids, tile_ids) => {
                if tile_ids.len() != 1 {
                    panic!("only supports one tile");
                }

                game_state.edit::<StatePositionPlayer>(|y| {
                    for x in player_ids {
                        let Some(position) = y.positions.get_mut(x) else { return };
                        position.0 = tile_ids[0].x;
                        position.1 = tile_ids[0].y;
                    }
                });
            }
            _ => {}
        }
    }
}
