use crate::{
    Collections::{event_queue::EventQueue2, game_state::GameState},
    IO::AssetLoader::AssetLoader,
};
use hecs::World;
use intertrait::CastFrom;

pub trait ECSSystemEventless: CastFrom {
    // data
    fn order(&self, _: &GameState, _: &World) -> i32 {
        0
    }
    fn is_enabled(&mut self, game_state: &mut GameState, world: &mut World) -> bool;

    // init
    fn init(&mut self, _: &mut GameState, _: &mut World, _: &mut EventQueue2, _: &mut AssetLoader) {}

    // events
    fn debug(&mut self, _: &mut GameState, _: &mut World, _: &mut EventQueue2) {}

    // life
    fn enable(&mut self, _: &mut GameState, _: &mut World, _: &mut EventQueue2) {}
    fn disable(&mut self, _: &mut GameState, _: &mut World, _: &mut EventQueue2) {}

    // tick
    fn will_tick(&mut self, _: &mut GameState, _: &mut World, _: &mut EventQueue2) {}
    fn tick(&mut self, _: &mut GameState, _: &mut World, _: &mut EventQueue2) {}
    fn did_tick(&mut self, _: &mut GameState, _: &mut World, _: &mut EventQueue2) {}
}
