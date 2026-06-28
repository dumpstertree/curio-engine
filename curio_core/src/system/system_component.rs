use crate::input::axis_code::AxisCode;
use crate::{ButtonCode, ButtonPressed, ComponentState, Nerve, TabState};
use crate::{EngineCommands, Ledger};
use crate::{Formation, Vector3};
use egui_wgpu::wgpu::{CommandEncoder, Texture, TextureView};
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
    fn input_button(&mut self, _ledger: &mut Vec<Ledger>, _key_code: ButtonCode, _val: ButtonPressed) {}

    // application
    fn application_quit(&mut self) {}
    fn application_resize(&mut self, _: f32, _: f32) {}

    // raw
    fn raw_event(&mut self, _: WindowEvent) {}
    fn set_game_mode(&mut self, _ledger: &mut Vec<Ledger>, _game_mode: &Formation) {}

    // state
    fn get_state(&self, _ledger: &Vec<Ledger>) -> Vec<(String, TabState)> {
        vec![]
    }
    fn get_facets(&self) -> Vec<ComponentState> {
        vec![]
    }

    fn render(&mut self, output_texture: &Texture, output_view: &TextureView, command_encoder: &mut CommandEncoder, ledger: &mut Vec<Ledger>, event_queue: &mut Vec<Nerve>) {}
}
