use core::collections::{event_queue::EventQueue, game_state::GameState};
use hecs::World;

use crate::{traits::scope::Scope, context_3d::Context3D};

pub trait Habit: Scope {
    // data
    fn order(&self, _: &GameState, _: &World) -> i32 {
        0
    }
    // init
    fn init(&mut self, _: &mut GameState, _: &mut Context3D, _: &mut EventQueue) {}

    // events
    fn debug(&mut self, _: &mut GameState, _: &mut Context3D, _: &mut EventQueue) {}

    // life
    fn enable(&mut self, _: &mut GameState, _: &mut Context3D, _: &mut EventQueue) {}
    fn disable(&mut self, _: &mut GameState, _: &mut Context3D, _: &mut EventQueue) {}

    // tick
    fn will_tick(&mut self, _: &mut GameState, _: &mut Context3D, _: &mut EventQueue) {}
    fn tick(&mut self, _: &mut GameState, _: &mut Context3D, _: &mut EventQueue) {}
    fn did_tick(&mut self, _: &mut GameState, _: &mut Context3D, _: &mut EventQueue) {}
}
