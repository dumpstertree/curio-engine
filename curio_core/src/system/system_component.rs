use crate::collections::game_state::GameState;
use crate::collections::key_state::KeyState;
use crate::collections::vector3::Vector3;
use crate::dumpster_engine::GameMode;
use crate::events::engine_commands::EngineCommands;
use crate::input::axis_code::AxisCode;
use crate::{collections::event_queue::EventQueue, input::key_code::ButtonCode};
use winit::event::WindowEvent;

pub trait SystemComponent {
    //
    fn order(&self) -> i32 {
        0
    }
    fn refresh(&mut self, _game_state: &mut Vec<GameState>, _event_queue: &mut Vec<EventQueue>) -> Vec<EngineCommands> {
        vec![]
    }

    // lifecycle
    fn init(&mut self, _game_state: &mut Vec<GameState>) {}
    fn tick(&mut self, _game_state: &mut Vec<GameState>, _event_queue: &mut Vec<EventQueue>) {}
    fn debug(&mut self, _game_state: &mut Vec<GameState>, _event_queue: &mut Vec<EventQueue>) {}

    // input
    fn input_axis(&mut self, _game_statee: &mut Vec<GameState>, _axis_code: AxisCode, _val: Vector3) {}
    fn input_button(&mut self, _game_state: &mut Vec<GameState>, _key_code: ButtonCode, _val: KeyState) {}

    // application
    fn application_quit(&mut self) {}
    fn application_resize(&mut self, _: f32, _: f32) {}

    // raw
    fn raw_event(&mut self, _: WindowEvent) {}
    fn set_game_mode(&mut self, _game_state: &mut Vec<GameState>, _game_mode: &GameMode) {}
}
