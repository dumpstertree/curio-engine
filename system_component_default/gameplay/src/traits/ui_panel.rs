use curio_core::{AxisCode, AxisState, ButtonCode, ButtonPressed};

use crate::traits_internal::ui_common::UICommon;

pub trait UIPanel: UICommon {
    fn input_button(&mut self, button: ButtonCode, state: ButtonPressed);
    fn input_axis(&mut self, axis: AxisCode, state: AxisState);
}
