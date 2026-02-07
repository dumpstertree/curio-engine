use curio_core::built_in::record::sys_record_input::SysRecordInput;
use curio_core::collections::event_queue::EventQueue;
use curio_core::collections::game_mode::GameMode;
use curio_core::collections::game_state::GameState;
use curio_core::{AxisCode, ButtonCode, InputMapping, PlayerInputSnapshot, Vector2, Vector3};
use std::collections::HashMap;

use curio_core::{
    collections::key_state::KeyState,
    system::{system_component::SystemComponent, system_components::system_component_input::SystemComponentInput},
};

pub struct SystemComponentDefaultInput {
    mappings_is_dirty: bool,
    state_axis: HashMap<AxisCode, Vector2>,
    state_button: HashMap<ButtonCode, bool>,
    active_mappings: Vec<Vec<InputMapping>>,
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

    fn tick(&mut self, game_state: &mut Vec<GameState>, _: &mut Vec<EventQueue>) {
        let mut cur_state = 0;
        // iterate over each
        for game_state in game_state {
            //
            game_state.edit::<SysRecordInput>(|x| {
                // if mismatched map length we need to rebuild - this is actually an issue because what if same amount
                if self.mappings_is_dirty {
                    // clear old
                    x.mapped.clear();

                    // create new
                    for mapping in &self.active_mappings[cur_state] {
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
            cur_state += 1;
        }
        // turn off flag
        self.mappings_is_dirty = false;
    }
    fn input_axis(&mut self, _: &mut Vec<GameState>, code: AxisCode, val: Vector3) {
        self.state_axis.insert(code, val.to_vector2());
    }
    fn input_button(&mut self, _: &mut Vec<GameState>, code: ButtonCode, val: KeyState) {
        self.state_button.insert(code, val == KeyState::Down);
    }
    fn set_game_mode(&mut self, _game_state: &mut Vec<GameState>, game_mode: &GameMode) {
        let mut active_mappings = vec![];
        for game_instance in &game_mode.game_instances {
            active_mappings.push(game_instance.input_mappings.clone());
            println!("set game mode with num inputs {}", game_instance.input_mappings.len());
        }
        self.active_mappings = active_mappings;
        self.mappings_is_dirty = true;
    }
}
