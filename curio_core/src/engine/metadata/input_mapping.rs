use crate::{input::axis_code::AxisCode, ButtonCode};

/// A mapping of inputs for one user
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize, Hash, Eq, PartialEq)]
pub struct InputMapping {
    pub mapping_button: Vec<(String, ButtonCode)>,
    pub mapping_axis: Vec<(String, AxisCode)>,
}
impl InputMapping {
    pub fn new(button: Vec<(String, ButtonCode)>, axis: Vec<(String, AxisCode)>) -> InputMapping {
        InputMapping { mapping_button: button, mapping_axis: axis }
    }
}
