//! The "Import from Penpot" dialog, opened from the Assets context menu (`TreeAction::PenpotDialog`).
//! Same shape as `figma_tab.rs`: inputs bind into `PenpotState`, the job reports via `poll_penpot`.

use eframe::egui::{self, RichText, Window};

use crate::state::EditorState;
use crate::theme;

pub fn show_dialog(ctx: &egui::Context, state: &mut EditorState) {
    if !state.asset.penpot.dialog_open {
        return;
    }

    Window::new("Import from Penpot").collapsible(false).resizable(true).show(ctx, |ui| {
        ui.label(RichText::new("Paste the workspace URL from your browser (self-hosted instances work: the API host is taken from it) and an access token from Your account > Access tokens, then hit Fetch + Import. Each top-level board becomes a .comp; a page-id in the URL limits the import to that page. Files land in: ").color(theme::TEXT_PRIMARY));
        ui.label(&state.asset.penpot.target_dir);

        ui.separator();

        ui.label("Workspace URL");
        ui.add(egui::TextEdit::singleline(&mut state.asset.penpot.url).desired_width(ui.available_width().max(40.0)));
        ui.label("Access token");
        ui.add(egui::TextEdit::singleline(&mut state.asset.penpot.token).password(true).desired_width(ui.available_width().max(40.0)));

        ui.separator();

        if ui.button("Fetch + Import").clicked() {
            state.asset.penpot.start();
        }

        if !state.asset.penpot.progress.is_empty() {
            ui.label(RichText::new(state.asset.penpot.progress.join("\n")).monospace().color(theme::TEXT_SECONDARY));
        }
        if let Some(summary) = state.asset.penpot.summary.clone() {
            ui.label(RichText::new(summary).monospace().color(theme::GREEN));
        }
        if let Some(error) = state.asset.penpot.error.clone() {
            ui.label(RichText::new(error).monospace().color(theme::RED));
        }

        // A running job keeps going after Close; only its report is dropped.
        if ui.button("Close").clicked() {
            state.asset.penpot.dialog_open = false;
            state.asset.penpot.progress = Vec::new();
            state.asset.penpot.summary = None;
            state.asset.penpot.error = None;
        }
    });
}
