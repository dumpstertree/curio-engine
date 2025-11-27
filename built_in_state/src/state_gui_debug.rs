use core::{
    collections::{color::Color, event_queue::EventQueue, game_state::GameState, vector3::Vector3},
    events::engine_commands::EngineCommands,
    extensions::extensions_f32::ExtensionsF32,
    system::system_game_state::IState,
};
use std::hash::Hash;

use macro_state::global_state;

use crate::{
    state_debug::StateDebug,
    state_gui::{GuiElement, GuiWindow},
};

#[global_state]
pub struct GUIStateDebug {
    pub color: Color,
    pub size: f32,
    contents: Vec<String>,
}
impl Hash for GUIStateDebug {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.color.hash(state);
        self.size.hash(state);
        self.contents.hash(state);
    }
}
impl Eq for GUIStateDebug {}

impl GUIStateDebug {
    pub fn append(&mut self, content: String) {
        self.contents.push(content);
    }
    pub fn finalize(&self, game_state: &mut GameState) -> GuiWindow {
        let is_paused = game_state.get::<StateDebug>().is_paused;
        let mut window = GuiWindow::new("debug".to_string(), Vector3::new(10.0, 10.0, 0.0), Vector3::zero());
        window.add(GuiElement::new_text_button(if is_paused { "Play" } else { "Pause" }, GUIStateDebug::pause_on_click));
        for x in &self.contents {
            window.add(GuiElement::new_label(x.clone(), self.size, self.color.clone()));
        }
        window
    }
    pub fn clear(&mut self) {
        self.contents.clear();
    }

    pub fn default() -> GUIStateDebug {
        GUIStateDebug {
            contents: Vec::new(),
            color: Color::new_hex("#f4ac62"),
            size: 18.0,
        }
    }
    fn pause_on_click(game_state: &mut GameState, event_queue: &mut EventQueue) {
        // event_queue.enqueue_event(EngineCommands::SetPauseMode(!game_state.get::<StateDebug>().is_paused));
    }
}
impl IState for GUIStateDebug {
    fn id() -> i32 {
        902945
    }
}
