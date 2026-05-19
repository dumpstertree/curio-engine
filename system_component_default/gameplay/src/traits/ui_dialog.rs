use crate::traits_internal::ui_common::UICommon;
use curio_core::{AxisCode, AxisState, ButtonCode, ButtonState};

pub trait UIDialog: UICommon {
    fn input_button(button: ButtonCode, state: ButtonState);
    fn input_axis(axis: AxisCode, state: AxisState);
}
