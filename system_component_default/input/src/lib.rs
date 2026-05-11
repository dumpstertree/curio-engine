use curio_core::built_in::record::sys_record_input::SysRecordInput;
use curio_core::collections::event_queue::Nerve;
use curio_core::collections::game_mode::GameMode;
use curio_core::collections::ledger::Ledger;
use curio_core::{Application, AxisCode, ButtonCode, InputMapping, KeyState, PlayerInputSnapshot, Severity, Vector2, Vector3};
use std::collections::HashMap;

use curio_core::system::system_component::SystemComponent;

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
// impl SystemComponentInput for SystemComponentDefaultInput {}
impl SystemComponent for SystemComponentDefaultInput {
    fn order(&self) -> i32 {
        1000
    }

    fn name(&self) -> String {
        "Input".to_owned()
    }
    fn tick(&mut self, ledger: &mut Vec<Ledger>, _: &mut Vec<Nerve>) {
        let mut cur_state = 0;
        // iterate over each
        for ledger in ledger {
            //
            ledger.write::<SysRecordInput>(|x| {
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
    fn input_axis(&mut self, _: &mut Vec<Ledger>, code: AxisCode, val: Vector3) {
        self.state_axis.insert(code, val.to_vector2());
    }
    fn input_button(&mut self, _: &mut Vec<Ledger>, code: ButtonCode, val: KeyState) {
        self.state_button.insert(code, val == KeyState::Down);
    }
    fn set_game_mode(&mut self, ledger: &mut Vec<Ledger>, game_mode: &GameMode) {
        let mut active_mappings = vec![];
        let mut index = 0;
        for game_instance in &game_mode.game_instances {
            active_mappings.push(game_instance.input_mappings.clone());
            ledger[index].log(Severity::Info, &format!("Set num of Inputs: {}", game_instance.input_mappings.len()));
            index += 1;
        }
        self.active_mappings = active_mappings;
        self.mappings_is_dirty = true;
    }
}
