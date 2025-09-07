use crate::{
    game_events::GameEvents,
    state::{state_energy::StateEnergy, state_turn::StateTurn},
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
pub struct ECSSystemGameTurnBegin {}
impl ECSSystemEventless for ECSSystemGameTurnBegin {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}
#[global_ecs_system_event_reciever(GameEvents)]
impl ecs_event_reciever::EventReciever<GameEvents> for ECSSystemGameTurnBegin {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut World, _: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::TurnBegin(id) => {
                // end this turn
                println!("Instance: {}. Begin Turn {}", game_state.instance_id, id);

                game_state.edit::<StateTurn>(|x| {
                    x.active_instance_id = *id;
                });
                // update energy
                game_state.edit::<StateEnergy>(|x| {
                    let cur = x.all_players[id];
                    x.all_players.insert(*id, (cur.1, cur.1));
                });
            }
            _ => {}
        }
    }
}
