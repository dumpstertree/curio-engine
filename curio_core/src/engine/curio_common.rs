use crate::{input::axis_code::AxisCode, ButtonCode, ButtonPressed, Vector3};

/// Base trait that an object needs to implement to be a Curio.
/// Handles propogating external events into the Curio such as Application, Window and Input events.
pub trait CurioCommon {
    // application
    fn update(&mut self) {}
    // window
    fn window_opened(&mut self) {}
    fn window_closed(&mut self) {}
    fn window_resized(&mut self) {}
    fn window_moved(&mut self) {}
    fn window_focused(&mut self, _is_focused: bool) {}
    fn window_occluded(&mut self, _is_occluded: bool) {}
    // input
    fn input_axis(&mut self, _axis: AxisCode, _state: Vector3) {}
    fn input_button(&mut self, _button: ButtonCode, _state: ButtonPressed) {}
}
