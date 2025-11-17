use crate::input::{axis_code::AxisCode, key_code::ButtonCode};

/// A mapping of inputs for one player
#[derive(Clone, serde::Serialize, serde::Deserialize, Hash, Eq, PartialEq)]
pub struct InputMapping {
    pub mapping_button: Vec<(String, ButtonCode)>,
    pub mapping_axis: Vec<(String, AxisCode)>,
}
impl InputMapping {
<<<<<<< HEAD
    pub fn new(button: Vec<(String, KeyCode)>, axis: Vec<(String, AxisCode)>) -> InputMapping {
        InputMapping { mapping_button: button, mapping_axis: axis }
=======
    pub fn new(button: Vec<(String, ButtonCode)>, axis: Vec<(String, AxisCode)>) -> InputMapping {
        InputMapping {
            mapping_button: button,
            mapping_axis: axis,
        }
>>>>>>> refs/remotes/origin/main
    }
}
