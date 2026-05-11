use crate::built_in::stimulant::engine_commands::EngineCommands;
use crate::collections::game_mode::GameMode;
use crate::collections::ledger::Ledger;
use crate::input::axis_code::AxisCode;
use crate::input::key_state::KeyState;
use crate::Vector3;
use crate::{collections::event_queue::Nerve, input::key_code::ButtonCode};
use winit::event::WindowEvent;

pub trait SystemComponent {
    fn name(&self) -> String;
    //
    fn order(&self) -> i32 {
        0
    }
    fn refresh(&mut self, _ledger: &mut Vec<Ledger>, _event_queue: &mut Vec<Nerve>) -> Vec<EngineCommands> {
        vec![]
    }

    // lifecycle
    fn init(&mut self, _ledger: &mut Vec<Ledger>) {}
    fn tick(&mut self, _ledger: &mut Vec<Ledger>, _event_queue: &mut Vec<Nerve>) {}
    fn debug(&mut self, _ledger: &mut Vec<Ledger>, _event_queue: &mut Vec<Nerve>) {}

    // input
    fn input_axis(&mut self, _ledgere: &mut Vec<Ledger>, _axis_code: AxisCode, _val: Vector3) {}
    fn input_button(&mut self, _ledger: &mut Vec<Ledger>, _key_code: ButtonCode, _val: KeyState) {}

    // application
    fn application_quit(&mut self) {}
    fn application_resize(&mut self, _: f32, _: f32) {}

    // raw
    fn raw_event(&mut self, _: WindowEvent) {}
    fn set_game_mode(&mut self, _ledger: &mut Vec<Ledger>, _game_mode: &GameMode) {}
}
