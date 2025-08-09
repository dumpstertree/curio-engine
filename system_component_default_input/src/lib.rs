use std::collections::HashMap;

use core::Collections::event_queue::EventQueue2;
use core::Collections::game_state::GameState;
use core::Collections::vector3::Vector3;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

use core::{
    Collections::key_state::KeyState,
    system::{
        system_component::SystemComponent, system_components::system_component_input::SystemComponentInput,
        system_game_states::state_input::InputState,
    },
};

pub struct SystemComponentDefaultInput {
    cursor_pos: Vector3,
    input_state_keyboard: HashMap<KeyCode, bool>,
    input_state_cursor: HashMap<MouseButton, bool>,
}

impl SystemComponentDefaultInput {
    pub fn new() -> Box<SystemComponentDefaultInput> {
        Box::new(SystemComponentDefaultInput {
            cursor_pos: Vector3::zero(),
            input_state_keyboard: HashMap::new(),
            input_state_cursor: HashMap::new(),
        })
    }
}
impl SystemComponentInput for SystemComponentDefaultInput {}
impl SystemComponent for SystemComponentDefaultInput {
    fn order(&self) -> i32 {
        1000
    }
    fn init(&mut self, gs: &mut GameState) {
        println!("init input");
    }
    fn tick(&mut self, game_state: &mut GameState, system_event_queue: &mut EventQueue2) {
        game_state.edit::<InputState>(|x| {
            // update cursor
            x.cursor.update(self.cursor_pos);

            // update keys
            for i in self.input_state_cursor.iter() {
                let key = i.0;
                let key_state = i.1;
                match key {
                    MouseButton::Left => x.cursor_primary.update(key_state),
                    _ => {}
                }
            }
            // update keys
            for i in self.input_state_keyboard.iter() {
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
    }
    fn input_mouse_position(&mut self, game_state: &mut GameState, position: Vector3) {
        self.cursor_pos = position;
    }
    fn input_keyboard(&mut self, game_state: &mut GameState, key: KeyCode, key_state: KeyState) {
        self.input_state_keyboard
            .insert(key, key_state == KeyState::Down);
    }
    fn input_mouse(&mut self, key: MouseButton, key_state: KeyState) {
        self.input_state_cursor
            .insert(key, key_state == KeyState::Down);
    }
}
