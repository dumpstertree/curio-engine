use crate::{
    system::{system_component::ISystemComponent, system_components::gameplay_component::IGameplayComponent},
    Collections::{event_queue::EventQueue2, game_state::GameState, vector3::Vector3},
    IO::AssetLoader::AssetLoader,
};
use hecs::World;
use intertrait::{cast::CastMut, CastFrom};

pub trait ECSSystemEventless: CastFrom {
    // data
    fn order(&self, game_state: &GameState, world: &World) -> i32 {
        0
    }
    fn is_enabled(&mut self, game_state: &mut GameState, world: &mut World) -> bool;

    // init
    fn init(&mut self, state: &mut GameState, world: &mut World, events: &mut EventQueue2, asset_loader: &mut AssetLoader) {}

    // events
    fn debug(&mut self, state: &mut GameState, world: &mut World, events: &mut EventQueue2) {}
    fn enable(&mut self, state: &mut GameState, world: &mut World, events: &mut EventQueue2) {}
    fn disable(&mut self, state: &mut GameState, world: &mut World, events: &mut EventQueue2) {}
    fn will_tick(&mut self, state: &mut GameState, world: &mut World, events: &mut EventQueue2) {}
    fn tick(&mut self, state: &mut GameState, world: &mut World, events: &mut EventQueue2) {}
    fn did_tick(&mut self, state: &mut GameState, world: &mut World, events: &mut EventQueue2) {}
}
