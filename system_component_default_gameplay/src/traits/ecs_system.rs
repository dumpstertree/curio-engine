use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};

// use crate::{
//     collections::{event_queue::EventQueue, game_state::GameState},
//     dumpster_engine::NetworkModes,
//     gameplay::world_context::WorldContext,
// };
use hecs::World;
use intertrait::CastFrom;

use crate::world_context::WorldContext;

pub trait ECSSystemEventless: CastFrom {
    // data
    fn order(&self, _: &GameState, _: &World) -> i32 {
        0
    }
    fn is_enabled(&mut self, game_state: &mut GameState, world: &mut WorldContext) -> bool;
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::LocalPeer, NetworkModes::OnlineHost, NetworkModes::OnlinePeer]
    }

    // init
    fn init(&mut self, _: &mut GameState, _: &mut WorldContext, _: &mut EventQueue) {}

    // events
    fn debug(&mut self, _: &mut GameState, _: &mut WorldContext, _: &mut EventQueue) {}

    // life
    fn enable(&mut self, _: &mut GameState, _: &mut WorldContext, _: &mut EventQueue) {}
    fn disable(&mut self, _: &mut GameState, _: &mut WorldContext, _: &mut EventQueue) {}

    // tick
    fn will_tick(&mut self, _: &mut GameState, _: &mut WorldContext, _: &mut EventQueue) {}
    fn tick(&mut self, _: &mut GameState, _: &mut WorldContext, _: &mut EventQueue) {}
    fn did_tick(&mut self, _: &mut GameState, _: &mut WorldContext, _: &mut EventQueue) {}
}
