use std::{string, sync::Arc};

use crate::{
    random::Random,
    system::{
        system_components::gameplay_components::gameplay_component_default::{EngineCommands, EventQueue},
        system_game_state::IState,
    },
    Collections::{
        game_state::{self, GameState},
        vector3::Vector3,
        Color::Color,
    },
};

#[derive(Clone)]
pub struct GUIState {
    pub guis: Vec<GuiWindow>,
}
impl GUIState {
    pub fn default() -> GUIState {
        GUIState { guis: Vec::new() }
    }
}
impl IState<GUIState> for GUIState {
    fn default() -> GUIState {
        GUIState::default()
    }

    fn id() -> i32 {
        690345
    }
}
#[derive(Clone)]
pub struct LabelDesc {
    pub contents: String,
    pub font_size: f32,
    pub color: Color,
}
#[derive(Clone)]
pub struct ButtonDesc {
    pub contents: String,
    pub on_click: fn(game_state: &mut GameState, &mut EventQueue<EngineCommands>),
}

#[derive(Clone)]
pub enum GuiElementTypes {
    Rectangle,
    Ellipse,
    Label(LabelDesc),
    Button(ButtonDesc),
}
#[derive(Clone)]
pub struct GuiElement {
    pub gui_type: GuiElementTypes,
}
impl GuiElement {
    pub fn new_rectangle() -> GuiElement {
        GuiElement {
            gui_type: GuiElementTypes::Rectangle,
        }
    }
    pub fn new_ellipse() -> GuiElement {
        GuiElement {
            gui_type: GuiElementTypes::Ellipse,
        }
    }
    pub fn new_label(label: String, size: f32, color: Color) -> GuiElement {
        GuiElement {
            gui_type: GuiElementTypes::Label(LabelDesc {
                contents: label,
                font_size: size,
                color: color,
            }),
        }
    }
    pub fn new_text_button(contents: &str, on_click: fn(&mut GameState, &mut EventQueue<EngineCommands>)) -> GuiElement {
        GuiElement {
            gui_type: GuiElementTypes::Button(ButtonDesc {
                contents: contents.to_string(),
                on_click: on_click,
            }),
        }
    }

    pub fn size_mode_x(&self) {}
    pub fn size_mode_y(&self) {}
    pub fn children(&self, children: Vec<GuiElement>) {}
}

#[derive(Clone)]
pub struct GuiWindow {
    pub instance_id: String,
    pub position: Vector3,
    pub anchor: Vector3,
    pub children: Vec<GuiElement>,
}
impl GuiWindow {
    pub fn new(id: String, position: Vector3, anchor: Vector3) -> GuiWindow {
        GuiWindow {
            position,
            anchor,
            children: Vec::new(),
            instance_id: id,
        }
    }
    pub fn add(&mut self, element: GuiElement) -> &mut GuiWindow {
        self.children.push(element);
        self
    }
}
