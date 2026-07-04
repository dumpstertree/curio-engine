use crate::state::{CompileStatus, EditorMode, EditorState};
use crate::theme;
use curio_core::ObjectState;
use eframe::egui::{RichText, Ui};

fn count_objects(objects: &[ObjectState]) -> usize {
    objects.iter().map(|o| 1 + count_objects(&o.children)).sum()
}

pub fn show(ui: &mut Ui, state: &EditorState) {
    ui.horizontal(|ui| {
        match state.mode {
            EditorMode::Playing => {
                ui.label(RichText::new("\u{25CF}").color(theme::PLAY));
                ui.label(RichText::new("Playing").color(theme::TEXT_PRIMARY));
            }
            EditorMode::Paused => {
                ui.label(RichText::new("\u{23F8} Paused").color(theme::PAUSE));
            }
            EditorMode::Stopped => {
                ui.label(RichText::new("\u{25A0} Stopped").color(theme::TEXT_SECONDARY));
            }
        }

        if state.compile_status == CompileStatus::Compiling {
            ui.separator();
            ui.label(RichText::new("Compiling\u{2026}").color(theme::ACCENT));
        }
        if state.compile_status == CompileStatus::Error {
            ui.separator();
            ui.label(RichText::new("\u{2715} Compile error").color(theme::RED));
        }

        if let Some(tgs) = &state.tab_group_state {
            let instance_count = tgs.id_for_tabs.len();
            let node_count: usize = tgs.id_for_tabs.values().flatten().map(|tab| count_objects(&tab.objects)).sum();

            if node_count > 0 {
                ui.separator();
                ui.label(RichText::new(format!("{node_count} objects")).color(theme::TEXT_SECONDARY));
            }
            if instance_count > 0 {
                ui.separator();
                ui.label(RichText::new(format!("{instance_count} instance{}", if instance_count == 1 { "" } else { "s" })).color(theme::TEXT_SECONDARY));
            }
        }

        ui.with_layout(eframe::egui::Layout::right_to_left(eframe::egui::Align::Center), |ui| {
            ui.label(RichText::new("curio engine").color(theme::TEXT_MUTED));
        });
    });
}
