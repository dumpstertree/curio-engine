use crate::{input::axis_code::AxisCode, AxisState, ButtonCode, ButtonState, Vector2};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::Hash};

// A snapshot of the current state of the input regardless of mappings
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputRaw {
    button_default: ButtonState,
    axis_default: AxisState,
    button: HashMap<ButtonCode, ButtonState>,
    axis: HashMap<AxisCode, AxisState>,
}
impl Hash for InputRaw {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.button_default.hash(state);
        self.axis_default.hash(state);

        let mut keys: Vec<&ButtonCode> = self.button.keys().collect();
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
impl InputRaw {
    pub fn new() -> InputRaw {
        InputRaw {
            button_default: ButtonState::default(),
            axis_default: AxisState::default(),
            button: HashMap::new(),
            axis: HashMap::new(),
        }
    }
    pub fn update(&mut self, raw_buttons: &HashMap<ButtonCode, bool>, raw_axis: &HashMap<AxisCode, Vector2>) {
        for map in raw_buttons {
            if !self.button.contains_key(map.0) {
                self.button.insert(*map.0, ButtonState::default());
            }
            self.button.get_mut(map.0).unwrap().update(map.1);
        }
        for map in raw_axis {
            if !self.axis.contains_key(map.0) {
                self.axis.insert(*map.0, AxisState::default());
            }
            self.axis.get_mut(map.0).unwrap().update(*map.1);
        }
    }

    pub fn get_button(&self, uid: &ButtonCode) -> &ButtonState {
        if let Some(result) = self.button.get(uid) {
            return result;
        }
        return &self.button_default;
    }
    pub fn get_axis(&self, uid: &AxisCode) -> &AxisState {
        if let Some(result) = self.axis.get(uid) {
            return result;
        }
        return &self.axis_default;
    }
}
