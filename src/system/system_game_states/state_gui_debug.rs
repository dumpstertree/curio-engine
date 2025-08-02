use crate::{
    system::{
        system_game_state::IState,
        system_game_states::state_gui::{GuiElement, GuiWindow},
    },
    Collections::{vector3::Vector3, Color::Color},
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
    pub fn finalize(&self) -> GuiWindow {
        let mut window = GuiWindow::new(String::from("debug"), Vector3::new(10.0, 10.0, 0.0), Vector3::zero());
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
}
impl IState<GUIState_Debug> for GUIState_Debug {
    fn default() -> GUIState_Debug {
        GUIState_Debug::default()
    }

    fn id() -> i32 {
        902945
    }
}
