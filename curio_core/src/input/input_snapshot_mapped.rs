use std::{collections::HashMap, hash::Hash};

use crate::{
    input::{axis_code::AxisCode, input_mapping::InputMapping, key_code::ButtonCode},
    InputAxisState, InputButtonState, Vector2,
};

// Result of testing raw input to mapped input
#[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct PlayerInputSnapshot {
    default_button: InputButtonState,
    defualt_axis: InputAxisState,
    map_button: HashMap<String, InputButtonState>,
    map_axis: HashMap<String, InputAxisState>,
    mapping: InputMapping,
}
impl Hash for PlayerInputSnapshot {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.default_button.hash(state);
        self.defualt_axis.hash(state);
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
