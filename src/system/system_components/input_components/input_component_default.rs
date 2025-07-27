use std::collections::HashMap;

use crate::Collections::game_state::GameState;
use winit::keyboard::KeyCode;

use crate::system::system_components::gameplay_components::gameplay_component_default::EngineCommands;
use crate::{
    system::{
        system_component::ISystemComponent,
        system_components::input_component::input_component,
        system_game_states::{state_input::InputState, state_time::TimeState},
    },
    Collections::key_state::KeyState,
};

pub struct InputComponentDefault {
    input_state: HashMap<KeyCode, bool>,
}

impl InputComponentDefault {
    pub fn new() -> InputComponentDefault {
        InputComponentDefault { input_state: HashMap::new() }
    }
}
impl input_component for InputComponentDefault {}
impl ISystemComponent for InputComponentDefault {
    fn order(&self) -> i32 {
        1000
    }
    fn init(&mut self, gs: &mut GameState) {
        println!("init input");
    }
    fn tick(&mut self, game_state: &mut GameState) -> &[EngineCommands] {
        let time = game_state.get_value2::<TimeState>();
        if !time.should_update {
            return &[];
        }

        game_state.edit::<InputState>(|x| {
            for i in self.input_state.iter() {
                let key = i.0;
                let key_state = i.1;
                match key {
                    KeyCode::KeyW => x.w.update(key_state),
                    KeyCode::KeyA => x.a.update(key_state),
                    KeyCode::KeyS => x.s.update(key_state),
                    KeyCode::KeyD => x.d.update(key_state),
                    KeyCode::Tab => x.tab.update(key_state),
                    KeyCode::KeyP => x.debug.update(key_state),
                    KeyCode::Escape => x.esc.update(key_state),
                    _ => {}
                }
            }
        });

        return &[];
    }
    fn input_keyboard(&mut self, game_state: &mut GameState, key: KeyCode, key_state: KeyState) {
        self.input_state.insert(key, key_state == KeyState::Down);
    }
}
