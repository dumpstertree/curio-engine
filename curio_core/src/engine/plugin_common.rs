use crate::input::axis_code::AxisCode;
use crate::{ButtonCode, ButtonPressed, ComponentState, Nerve, PluginState};
use crate::{Commands, Ledger};
use crate::{Formation, Vector3};
use egui_wgpu::wgpu::{CommandEncoder, Texture, TextureView};

/// Trait that must be implemented in order to be a valid Plugin
pub trait PluginCommon {
    /// Pretty name of this Plugin
    fn name(&self) -> String;

    /// Plugins will be sorted on creation by order and run in that order
    fn order(&self) -> i32 {
        0
    }
    /// Update the logic of the Plugin. This happens much more frequently than Tick. Update may return `Commands` for the Curio to parse next.
    fn update(&mut self, _ledger: &mut Vec<Ledger>, _event_queue: &mut Vec<Nerve>) -> Vec<Commands> {
        vec![]
    }

    /// Create a serializable version of the Plugin state for display or dumping. Optional.
    fn serializable(&self, _ledger: &Vec<Ledger>) -> Vec<(String, PluginState)> {
        vec![]
    }

    /// Create a serializable version of options that are available in the Plugin for display or dumping. Optional.
    fn peek(&self) -> Vec<ComponentState> {
        vec![]
    }

    /// Event called once before the first `tick` or `set_Formation`
    fn init(&mut self, _ledger: &mut Vec<Ledger>) {}

    /// Event called whenever the Application should update
    fn tick(&mut self, _ledger: &mut Vec<Ledger>, _event_queue: &mut Vec<Nerve>) {}

    /// Event called when floating point input is recieved
    fn input_axis(&mut self, _ledgere: &mut Vec<Ledger>, _axis_code: AxisCode, _val: Vector3) {}

    /// Event called when button input is recieved
    fn input_button(&mut self, _ledger: &mut Vec<Ledger>, _key_code: ButtonCode, _val: ButtonPressed) {}

    /// Event called when application has been instructed to quit
    fn application_quit(&mut self) {}

    /// Event called when application has been instructed to resize its window
    fn application_resize(&mut self, _: f32, _: f32) {}

    /// Event called that allows for more specific setup of the Plugin. This is called after Init but before first Tick.
    fn set_formation(&mut self, _ledger: &mut Vec<Ledger>, _game_mode: &Formation) {}

    /// Event called that to allow plugins to render to screen. These are run by their order value so Plugins run later are able to overwrite or edit earlier renders
    fn render(&mut self, _output_texture: &Texture, _output_view: &TextureView, _command_encoder: &mut CommandEncoder, _ledger: &mut Vec<Ledger>, _event_queue: &mut Vec<Nerve>) {}
}
