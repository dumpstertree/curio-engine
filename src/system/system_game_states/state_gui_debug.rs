use crate::{
    system::{
        system_components::gameplay_components::gameplay_component_default::{EngineCommands, EventQueue},
        system_game_state::IState,
        system_game_states::{
            state_debug::StateDebug,
            state_gui::{GuiElement, GuiWindow},
        },
    },
    Collections::{
        game_state::{self, GameState},
        vector3::Vector3,
        Color::Color,
    },
};

#[derive(Clone)]
pub struct GUIState_Debug {
    pub color: Color,
    pub size: f32,
    contents: Vec<String>,
}
impl GUIState_Debug {
    pub fn append(&mut self, content: String) {
        self.contents.push(content);
    }
    pub fn finalize(&self, game_state: &mut GameState) -> GuiWindow {
        let is_paused = game_state.get_value2::<StateDebug>().is_paused;
        let mut window = GuiWindow::new("debug".to_string(), Vector3::new(10.0, 10.0, 0.0), Vector3::zero());
        window.add(GuiElement::new_text_button(
            if is_paused { "Play" } else { "Pause" },
            GUIState_Debug::pause_on_click,
        ));
        for x in &self.contents {
            window.add(GuiElement::new_label(x.clone(), self.size, self.color.clone()));
        }
        window
    }
    pub fn clear(&mut self) {
        self.contents.clear();
    }

    pub fn default() -> GUIState_Debug {
        GUIState_Debug {
            contents: Vec::new(),
            color: Color::get_green(),
            size: 18.0,
        }
    }
    fn pause_on_click(game_state: &mut GameState, event_queue: &mut EventQueue<EngineCommands>) {
        event_queue.enqueue_event(EngineCommands::SetPauseMode(!game_state.get_value2::<StateDebug>().is_paused));
    }
}
impl IState<GUIState_Debug> for GUIState_Debug {
    fn default() -> GUIState_Debug {
        GUIState_Debug::default()
    }

    fn id() -> i32 {
        902945
    }
}
