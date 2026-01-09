use crate::traits_internal::ui_common::UICommon;
use core::{
    collections::{input_cursor::InputAxisState, key_state::KeyState},
    input::{axis_code::AxisCode, key_code::ButtonCode},
};

pub trait UIPanel: UICommon {
    fn input_button(&mut self, button: ButtonCode, state: KeyState);
    fn input_axis(&mut self, axis: AxisCode, state: InputAxisState);
}
