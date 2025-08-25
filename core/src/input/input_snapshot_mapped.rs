use std::collections::HashMap;

use crate::{
    collections::{input_button::InputButtonState, input_cursor::InputAxisState, vector2::Vector2},
    input::{axis_code::AxisCode, input_mapping::InputMapping, key_code::KeyCode},
};

// Result of testing raw input to mapped input
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct PlayerInputSnapshot {
    default_button: InputButtonState,
    defualt_axis: InputAxisState,
    map_button: HashMap<String, InputButtonState>,
    map_axis: HashMap<String, InputAxisState>,
    mapping: InputMapping,
}
impl PlayerInputSnapshot {
    pub fn new(mapping: InputMapping) -> PlayerInputSnapshot {
        let mut map_button = HashMap::new();
        for map in &mapping.mapping_button {
            map_button.insert(map.0.clone(), InputButtonState::default());
        }
        let mut map_axis = HashMap::new();
        for map in &mapping.mapping_button {
            map_axis.insert(map.0.clone(), InputAxisState::default());
        }

        PlayerInputSnapshot {
            default_button: InputButtonState::default(),
            defualt_axis: InputAxisState::default(),
            map_button,
            map_axis,
            mapping: mapping,
        }
    }
    /// Update the button and axis states based changes in system input. This should ONLY be called from the System Component
    pub fn update(&mut self, raw_buttons: &HashMap<KeyCode, bool>, raw_axis: &HashMap<AxisCode, Vector2>) {
        for mapping in &self.mapping.mapping_button {
            let result = raw_buttons.get(&mapping.1);
            if let Some(result) = result {
                self.map_button.get_mut(&mapping.0).unwrap().is_down = *result;
            } else {
                self.map_button.get_mut(&mapping.0).unwrap().is_down = false;
            }
        }
        for mapping in &self.mapping.mapping_axis {
            let result = raw_axis.get(&mapping.1);
            if let Some(result) = result {
                self.map_axis.get_mut(&mapping.0).unwrap().position = *result;
            } else {
                self.map_axis.get_mut(&mapping.0).unwrap().position = Vector2::zero();
            }
        }
    }
    /// Get the current state of a Button; if the UID provided is not part of the mapping returns NONE
    pub fn get_button(&self, uid: &str) -> Option<&InputButtonState> {
        self.map_button.get(uid)
    }
    /// Get the current state of a Button; if the UID provided is not part of the mapping returns a default version
    pub fn get_button_or_default(&self, uid: &str) -> &InputButtonState {
        let state = self.map_button.get(uid);
        if let Some(val) = state {
            return val;
        };
        return &self.default_button;
    }
    /// Get the current state of a Axis; if the UID provided is not part of the mapping returns NONE
    pub fn get_axis(&self, uid: &str) -> Option<&InputAxisState> {
        self.map_axis.get(uid)
    }
    /// Get the current state of a Axis; if the UID provided is not part of the mapping returns a default version
    pub fn get_axis_or_default(&self, uid: &str) -> &InputAxisState {
        let state = self.map_axis.get(uid);
        if let Some(val) = state {
            return val;
        };
        return &self.defualt_axis;
    }
}
