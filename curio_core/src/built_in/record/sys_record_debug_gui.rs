use std::{hash::Hash, sync::OnceLock};

static SYS_RECORD_ID: OnceLock<i32> = OnceLock::new();

use crate::{
    built_in::record::{
        sys_record_debug::SysRecordDebug,
        sys_record_gui::{GuiElement, GuiWindow},
    },
    collections::{event_queue::Nerve, ledger::Ledger},
    extensions::extensions_f32::ExtensionsF32,
    system::record_id::RecordId,
    Color, RecordCommon, Vector3,
};

#[derive(Default, PartialEq, Clone)]
pub struct SysRecordDebugGui {
    pub color: Color,
    pub size: f32,
    contents: Vec<String>,
}
impl Hash for SysRecordDebugGui {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.color.hash(state);
        self.size.hash(state);
        self.contents.hash(state);
    }
}
impl Eq for SysRecordDebugGui {}

impl SysRecordDebugGui {
    pub fn append(&mut self, content: String) {
        self.contents.push(content);
    }
    pub fn finalize(&self, ledger: &mut Ledger) -> GuiWindow {
        let is_paused = ledger.read::<SysRecordDebug>().is_paused;
        let mut window = GuiWindow::new("debug".to_string(), Vector3::new(10.0, 10.0, 0.0), Vector3::zero());
        window.add(GuiElement::new_text_button(if is_paused { "Play" } else { "Pause" }, SysRecordDebugGui::pause_on_click));
        for x in &self.contents {
            window.add(GuiElement::new_label(x.clone(), self.size, self.color.clone()));
        }
        window
    }
    pub fn clear(&mut self) {
        self.contents.clear();
    }

    pub fn default() -> SysRecordDebugGui {
        SysRecordDebugGui {
            contents: Vec::new(),
            color: Color::new_hex("#f4ac62"),
            size: 18.0,
        }
    }
    fn pause_on_click(_ledger: &mut Ledger, _event_queue: &mut Nerve) {
        // event_queue.enqueue_event(EngineCommands::SetPauseMode(!ledger.get::<StateDebug>().is_paused));
    }
}
impl RecordCommon for SysRecordDebugGui {
    fn id() -> i32 {
        *SYS_RECORD_ID.get_or_init(|| RecordId::of::<SysRecordDebugGui>())
    }
}
