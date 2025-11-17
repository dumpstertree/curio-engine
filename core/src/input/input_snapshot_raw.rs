use std::{collections::HashMap, hash::Hash};

use crate::{
    collections::{input_button::InputButtonState, input_cursor::InputAxisState, vector2::Vector2},
    input::{axis_code::AxisCode, key_code::ButtonCode},
};

// A snapshot of the current state of the input regardless of mappings
#[derive(Clone, Default, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct RawInputSnapshot {
    button_default: InputButtonState,
    axis_default: InputAxisState,
    button: HashMap<ButtonCode, InputButtonState>,
    axis: HashMap<AxisCode, InputAxisState>,
}
impl Hash for RawInputSnapshot {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.button_default.hash(state);
        self.axis_default.hash(state);

        let mut keys: Vec<&KeyCode> = self.button.keys().collect();
        keys.sort();
        keys.len().hash(state);
        for k in keys {
            k.hash(state);
            // unwrap is safe because k came from keys()
            self.button.get(k).unwrap().hash(state);
        }

        // same for map_axis
        let mut axis_keys: Vec<&AxisCode> = self.axis.keys().collect();
        axis_keys.sort();
        axis_keys.len().hash(state);
        for k in axis_keys {
            k.hash(state);
            self.axis.get(k).unwrap().hash(state);
        }
    }
}
impl RawInputSnapshot {
    pub fn new() -> RawInputSnapshot {
        RawInputSnapshot {
            button_default: InputButtonState::default(),
            axis_default: InputAxisState::default(),
            button: HashMap::new(),
            axis: HashMap::new(),
        }
    }
    pub fn update(&mut self, raw_buttons: &HashMap<ButtonCode, bool>, raw_axis: &HashMap<AxisCode, Vector2>) {
        for map in raw_buttons {
            if !self.button.contains_key(map.0) {
                self.button.insert(*map.0, InputButtonState::default());
            }
            self.button.get_mut(map.0).unwrap().update(map.1);
        }
        for map in raw_axis {
            if !self.axis.contains_key(map.0) {
                self.axis.insert(*map.0, InputAxisState::default());
            }
            self.axis.get_mut(map.0).unwrap().update(*map.1);
        }
    }

    pub fn get_button(&self, uid: &ButtonCode) -> &InputButtonState {
        if let Some(result) = self.button.get(uid) {
            return result;
        }
        return &self.button_default;
    }
    pub fn get_axis(&self, uid: &AxisCode) -> &InputAxisState {
        if let Some(result) = self.axis.get(uid) {
            return result;
        }
        return &self.axis_default;
    }
}
