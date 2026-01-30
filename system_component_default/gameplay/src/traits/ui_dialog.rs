use crate::traits_internal::ui_common::UICommon;
use curio_core::{
    collections::{input_button::InputButtonState, input_cursor::InputAxisState},
    input::{axis_code::AxisCode, key_code::ButtonCode},
};

pub trait UIDialog: UICommon {
    fn input_button(button: ButtonCode, state: InputButtonState);
    fn input_axis(axis: AxisCode, state: InputAxisState);
}
