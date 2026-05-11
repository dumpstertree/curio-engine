use std::{
    hash::{self, Hash},
    sync::OnceLock,
};

static SYS_RECORD_ID: OnceLock<i32> = OnceLock::new();

use crate::{
    collections::{event_queue::Nerve, ledger::Ledger},
    extensions::extensions_f32::ExtensionsF32,
    system::record_id::RecordId,
    Color, RecordCommon, Vector3,
};

#[derive(Default, Hash, Clone)]
pub struct SysRecordGui {
    pub guis: Vec<GuiWindow>,
}
impl SysRecordGui {
    pub fn default() -> SysRecordGui {
        SysRecordGui { guis: Vec::new() }
    }
}
impl RecordCommon for SysRecordGui {
    fn id() -> i32 {
        *SYS_RECORD_ID.get_or_init(|| RecordId::of::<SysRecordGui>())
    }
}
#[derive(Default, Clone, PartialEq)]
pub struct LabelDesc {
    pub contents: String,
    pub font_size: f32,
    pub color: Color,
}
impl Eq for LabelDesc {}
impl Hash for LabelDesc {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.contents.hash(state);
        self.font_size.hash(state);
        self.color.hash(state);
    }
}
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct ButtonDesc {
    pub contents: String,
    pub on_click: fn(ledger: &mut Ledger, &mut Nerve),
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum GuiElementTypes {
    Rectangle,
    Ellipse,
    Label(LabelDesc),
    Button(ButtonDesc),
}
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct GuiElement {
    pub gui_type: GuiElementTypes,
}
impl GuiElement {
    pub fn new_rectangle() -> GuiElement {
        GuiElement { gui_type: GuiElementTypes::Rectangle }
    }
    pub fn new_ellipse() -> GuiElement {
        GuiElement { gui_type: GuiElementTypes::Ellipse }
    }
    pub fn new_label(label: String, size: f32, color: Color) -> GuiElement {
        GuiElement {
            gui_type: GuiElementTypes::Label(LabelDesc { contents: label, font_size: size, color: color }),
        }
    }
    pub fn new_text_button(contents: &str, on_click: fn(&mut Ledger, &mut Nerve)) -> GuiElement {
        GuiElement {
            gui_type: GuiElementTypes::Button(ButtonDesc { contents: contents.to_string(), on_click: on_click }),
        }
    }

    pub fn size_mode_x(&self) {}
    pub fn size_mode_y(&self) {}
    pub fn children(&self, _: Vec<GuiElement>) {}
}

#[derive(Clone, Hash, PartialEq, Eq)]
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
