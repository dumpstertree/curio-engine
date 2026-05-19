use serde::{Deserialize, Serialize};

use crate::{input::axis_code::AxisCode, AxisState, ButtonCode, ButtonState, InputMapping, Vector2};
use std::{collections::HashMap, hash::Hash};

// Result of testing raw input to mapped input
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputMapped {
    default_button: ButtonState,
    default_axis: AxisState,
    map_button: HashMap<String, ButtonState>,
    map_axis: HashMap<String, AxisState>,
    mapping: InputMapping,
}
impl Hash for InputMapped {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.default_button.hash(state);
        self.default_axis.hash(state);
        self.mapping.hash(state);

        let mut keys: Vec<&String> = self.map_button.keys().collect();
        keys.sort();
        keys.len().hash(state);
        for k in keys {
            k.hash(state);
            // unwrap is safe because k came from keys()
            self.map_button.get(k).unwrap().hash(state);
        }

        // same for map_axis
        let mut axis_keys: Vec<&String> = self.map_axis.keys().collect();
        axis_keys.sort();
        axis_keys.len().hash(state);
        for k in axis_keys {
            k.hash(state);
            self.map_axis.get(k).unwrap().hash(state);
        }
    }
}
impl InputMapped {
    pub fn new(mapping: InputMapping) -> InputMapped {
        let mut map_button = HashMap::new();
        for map in &mapping.mapping_button {
            map_button.insert(map.0.clone(), ButtonState::default());
        }
        let mut map_axis = HashMap::new();
        for map in &mapping.mapping_button {
            map_axis.insert(map.0.clone(), AxisState::default());
        }

        InputMapped {
            default_button: ButtonState::default(),
            default_axis: AxisState::default(),
            map_button,
            map_axis,
            mapping: mapping,
        }
    }
    /// Update the button and axis states based changes in system input. This should ONLY be called from the System Component
    pub fn update(&mut self, raw_buttons: &HashMap<ButtonCode, bool>, raw_axis: &HashMap<AxisCode, Vector2>) {
        for mapping in &self.mapping.mapping_button {
            let result = raw_buttons.get(&mapping.1);
            if let Some(result) = result {
                self.map_button.get_mut(&mapping.0).unwrap().update(result);
            } else {
                self.map_button.get_mut(&mapping.0).unwrap().update(&false);
            }
        }
        for mapping in &self.mapping.mapping_axis {
            let result = raw_axis.get(&mapping.1);
            if let Some(result) = result {
                self.map_axis.get_mut(&mapping.0).unwrap().update(*result);
            } else {
                self.map_axis
                    .get_mut(&mapping.0)
                    .unwrap()
                    .update(Vector2::zero());
            }
        }
    }
    /// Get the current state of a Button; if the UID provided is not part of the mapping returns NONE
    pub fn get_button(&self, uid: &str) -> Option<&ButtonState> {
        self.map_button.get(uid)
    }
    /// Get the current state of a Button; if the UID provided is not part of the mapping returns a default version
    pub fn get_button_or_default(&self, uid: &str) -> &ButtonState {
        let state = self.map_button.get(uid);
        if let Some(val) = state {
            return val;
        };
        return &self.default_button;
    }
    /// Get the current state of a Axis; if the UID provided is not part of the mapping returns NONE
    pub fn get_axis(&self, uid: &str) -> Option<&AxisState> {
        self.map_axis.get(uid)
    }
    /// Get the current state of a Axis; if the UID provided is not part of the mapping returns a default version
    pub fn get_axis_or_default(&self, uid: &str) -> &AxisState {
        let state = self.map_axis.get(uid);
        if let Some(val) = state {
            return val;
        };
        return &self.default_axis;
    }
}
