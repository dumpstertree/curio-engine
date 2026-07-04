use crate::state::EditorState;
use eframe::egui::{self, Ui};

pub fn show(ui: &mut Ui, _state: &mut EditorState) {
    ui.horizontal(|ui| {
        ui.menu_button("File", |ui| {
            if ui.button("New          Ctrl+N").clicked() {
                ui.close_menu();
            }
            if ui.button("Load         Ctrl+O").clicked() {
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Undo         Ctrl+Z").clicked() {
                ui.close_menu();
            }
            if ui.button("Redo         Ctrl+Y").clicked() {
                ui.close_menu();
            }
        });
    });
}

/// Call once from `CurioEditorApp::update` to wire up Ctrl+N/O/Z/Y later —
/// left as a hook point since the original menu items are all placeholders
/// (no `onClick` behavior beyond closing the menu) in the source app too.
pub fn handle_shortcuts(_ctx: &egui::Context, _state: &mut EditorState) {}
