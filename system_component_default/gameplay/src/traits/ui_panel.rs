use curio_core::{AxisCode, ButtonCode, InputAxisState, collections::key_state::KeyState};

use crate::traits_internal::ui_common::UICommon;

pub trait UIPanel: UICommon {
    fn input_button(&mut self, button: ButtonCode, state: KeyState);
    fn input_axis(&mut self, axis: AxisCode, state: InputAxisState);
}
