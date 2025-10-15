use crate::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use hecs::World;
use intertrait::CastFrom;

pub trait ECSSystemEventless: CastFrom {
    // data
    fn order(&self, _: &GameState, _: &World) -> i32 {
        0
    }
    fn is_enabled(&mut self, game_state: &mut GameState, world: &mut World) -> bool;
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::LocalPeer, NetworkModes::OnlineHost, NetworkModes::OnlinePeer]
    }

    // init
    fn init(&mut self, _: &mut GameState, _: &mut World, _: &mut EventQueue) {}

    // events
    fn debug(&mut self, _: &mut GameState, _: &mut World, _: &mut EventQueue) {}

    // life
    fn enable(&mut self, _: &mut GameState, _: &mut World, _: &mut EventQueue) {}
    fn disable(&mut self, _: &mut GameState, _: &mut World, _: &mut EventQueue) {}

    // tick
    fn will_tick(&mut self, _: &mut GameState, _: &mut World, _: &mut EventQueue) {}
    fn tick(&mut self, _: &mut GameState, _: &mut World, _: &mut EventQueue) {}
    fn did_tick(&mut self, _: &mut GameState, _: &mut World, _: &mut EventQueue) {}
}
