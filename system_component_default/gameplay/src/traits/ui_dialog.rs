use crate::traits_internal::ui_common::UICommon;
use curio_core::{AxisCode, ButtonCode, InputAxisState, InputButtonState};

pub trait UIDialog: UICommon {
    fn input_button(button: ButtonCode, state: InputButtonState);
    fn input_axis(axis: AxisCode, state: InputAxisState);
}
