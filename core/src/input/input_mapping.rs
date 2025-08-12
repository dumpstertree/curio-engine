use crate::input::{axis_code::AxisCode, key_code::KeyCode};

/// A mapping of inputs for one player
#[derive(Clone)]
pub struct InputMapping {
    pub mapping_button: Vec<(String, KeyCode)>,
    pub mapping_axis: Vec<(String, AxisCode)>,
}
impl InputMapping {
    pub fn new(button: Vec<(String, KeyCode)>, axis: Vec<(String, AxisCode)>) -> InputMapping {
        InputMapping {
            mapping_button: button,
            mapping_axis: axis,
        }
    }
}
