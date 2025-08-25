use core::collections::game_state::GameState;
use core::collections::vector2::Vector2;
use core::collections::vector3::Vector3;
use core::input::axis_code::AxisCode;
use core::input::input_snapshot_mapped::PlayerInputSnapshot;
use core::input::key_code::KeyCode;
use core::{collections::event_queue::EventQueue, input::input_mapping::InputMapping};
use std::collections::HashMap;

use core::{
    collections::key_state::KeyState,
    system::{system_component::SystemComponent, system_components::system_component_input::SystemComponentInput},
};

use built_in_state::state_input::InputState;

pub struct SystemComponentDefaultInput {
    mappings_is_dirty: bool,
    state_axis: HashMap<AxisCode, Vector2>,
    state_button: HashMap<KeyCode, bool>,
    active_mappings: Vec<InputMapping>,
}

impl SystemComponentDefaultInput {
    pub fn new() -> Box<SystemComponentDefaultInput> {
        Box::new(SystemComponentDefaultInput {
            mappings_is_dirty: false,
            state_axis: HashMap::new(),
            state_button: HashMap::new(),
            active_mappings: Vec::new(),
        })
    }
}
impl SystemComponentInput for SystemComponentDefaultInput {}
impl SystemComponent for SystemComponentDefaultInput {
    fn order(&self) -> i32 {
        1000
    }

    fn tick(&mut self, game_state: &mut GameState, _: &mut EventQueue) {
        game_state.edit::<InputState>(|x| {
            // if mismatched map length we need to rebuild - this is actually an issue because what if same amount
            if self.mappings_is_dirty {
                // clear old
                x.mapped.clear();

                // create new
                for mapping in &self.active_mappings {
                    x.mapped.push(PlayerInputSnapshot::new(mapping.clone()));
                }
            }

            // update raw input to include changes
            x.raw.update(&self.state_button, &self.state_axis);

            // iterate over each mapped
            for i in 0..x.mapped.len() {
                // update mapped input to include changees
                x.mapped
                    .get_mut(i)
                    .unwrap()
                    .update(&self.state_button, &self.state_axis);
            }
        });
        // turn off flag
        self.mappings_is_dirty = false;
    }
    fn input_axis(&mut self, _: &mut GameState, code: AxisCode, val: Vector3) {
        self.state_axis.insert(code, val.to_vector2());
    }
    fn input_button(&mut self, _: &mut GameState, code: KeyCode, val: KeyState) {
        self.state_button.insert(code, val == KeyState::Down);
    }
    fn set_game_mode(&mut self, game_mode: &core::dumpster_engine::GameMode) {
        self.active_mappings = game_mode.input_mappings.clone();
        self.mappings_is_dirty = true;
    }
}
