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
    fn render(&mut self, game_state: &mut GameState) -> &[EngineCommands] {
        let time = game_state.get_value2::<TimeState>();
        if !time.should_update {
            return &[];
        }

        let mut input = game_state.get_value2::<InputState>();
        for i in self.input_state.iter() {
            let key = i.0;
            let key_state = i.1;
            match key {
                KeyCode::KeyW => input.w.update(key_state),
                KeyCode::KeyA => input.a.update(key_state),
                KeyCode::KeyS => input.s.update(key_state),
                KeyCode::KeyD => input.d.update(key_state),
                KeyCode::Tab => input.tab.update(key_state),
                KeyCode::KeyP => input.debug.update(key_state),
                KeyCode::Escape => input.esc.update(key_state),
                _ => {}
            }
        }

        game_state.set_value2::<InputState>(input);

        return &[];
    }
    fn input_keyboard(&mut self, game_state: &mut GameState, key: KeyCode, key_state: KeyState) {
        self.input_state.insert(key, key_state == KeyState::Down);
    }
}
