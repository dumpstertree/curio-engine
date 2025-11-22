use crate::{
    ai_resolver::run_ai,
    game_events::GameEvents,
    state::{state_energy::StateEnergy, state_turn::StateTurn},
};
use built_in_state::state_time::TimeState;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::{ecs_event_reciever, ecs_system::ECSSystemEventless},
};
use ecs_event::global_ecs_system_event_reciever;
use ecs_system::global_ecs_system;
use hecs::World;

use crate::{
    AssetMappingUIDs,
    ecs::components::{component_ball::ComponentBall, component_player::ComponentPlayer, component_view_player::ComponentViewPlayer},
    state::state_teams::{StateTeamAssignments, Teams},
};

#[global_ecs_system]
pub struct ECSSystemPeerStart {
    do_move: bool,
    lastmove: f64,
}
impl ECSSystemEventless for ECSSystemPeerStart {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalPeer, NetworkModes::OnlinePeer]
    }
    fn init(&mut self, game_state: &mut GameState, world: &mut World, _: &mut EventQueue) {
        println!("Instance: {}. Peer Init", game_state.instance_id);
    }
    fn enable(&mut self, game_state: &mut GameState, world: &mut World, _: &mut EventQueue) {
        println!("run on local enable");
    }
    fn tick(&mut self, game_state: &mut GameState, _: &mut World, events: &mut EventQueue) {
        if self.do_move && game_state.get::<TimeState>().unscaled_time - self.lastmove > 1.0 {
            println!("start new move");
            let e = run_ai(game_state);

            match &e {
                GameEvents::RequestTurnEnd(_) => self.do_move = false,
                _ => {}
            }

            println!("send event");
            events.enqueue_event(e);
            self.lastmove = game_state.get::<TimeState>().unscaled_time;

            self.do_move = false;
            game_state.get::<TimeState>().unscaled_time;
        }
    }
}
#[global_ecs_system_event_reciever(GameEvents)]
impl ecs_event_reciever::EventReciever<GameEvents> for ECSSystemPeerStart {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut World, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::DidTurnBegin(id) => {
                println!("did begine!!!!");
                self.do_move = true;
                self.lastmove = game_state.get::<TimeState>().unscaled_time;
            }
            _ => {}
        }
    }
}
