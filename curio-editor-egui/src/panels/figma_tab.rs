//! The "Import from Figma" dialog window — opened from the Assets tab's
//! right-click context menu (`panels/asset_tree.rs`'s `creation_menu` ->
//! `TreeAction::FigmaDialog`, which just sets `FigmaState::dialog_open` and
//! the target dir). `app.rs` calls `show_dialog` every frame inside the Asset
//! tab branch; it draws nothing while that flag is false, so the dialog floats
//! over whatever else the tab shows.
//!
//! The two inputs bind straight into `FigmaState`'s own Strings — egui's
//! `TextEdit::singleline` writes back into whatever it is handed, so the URL
//! and token double as "remember last import" values the next dialog reuses.
//! "Fetch + Import" starts the background job thread (`FigmaState::start`);
//! everything the thread reports arrives through `AssetState::poll_figma` and
//! shows up here as progress lines, so the UI never blocks on the network.

use eframe::egui::{self, RichText, Ui, Window};

use crate::theme;
use crate::state::EditorState;

pub fn show_dialog(ctx: &egui::Context, state: &mut EditorState) {
    if !state.asset.figma.dialog_open {
        return;
    }

    Window::new("Import from Figma")
        .collapsible(false)
        .resizable(true)
        .show(ctx, |ui| {

            ui.label(
                RichText::new(
                    "Paste a Figma file URL (figma.com/design/KEY/Name) and a personal access token, then hit Fetch + Import. Each top-level frame becomes a .comp with PNGs next to it; add ?node-id=1-23 to import one subtree. Files land in: "
                ).color(theme::TEXT_PRIMARY),
            );
            ui.label(&state.asset.figma.target_dir);

            ui.separator();

            // Mutating these fields directly — egui writes the edited text
            // back into the String it was given, so typing here *is* typing
            // into the state the job thread reads when it starts.
            ui.label("File URL");
            ui.add(
                egui::TextEdit::singleline(&mut state.asset.figma.url)
                    .desired_width(ui.available_width().max(40.0))
            );
            ui.label("Access token");
            ui.add(
                egui::TextEdit::singleline(&mut state.asset.figma.token)
                    .desired_width(ui.available_width().max(40.0))
            );

            ui.separator();

            if ui.button("Fetch + Import").clicked() {
                // Guarded inside `start` (no-op while a job is in flight or
                // with either input empty); the window manager keeps clicking
                // this again harmless.
                state.asset.figma.start();
            }

            if !state.asset.figma.progress.is_empty() {
                ui.label(
                    RichText::new(&state.asset.figma.progress.join("\n")).monospace()
                        .color(theme::TEXT_SECONDARY),
                );
            }
            if let Some(summary) = state.asset.figma.summary.clone() {
                ui.label(
                    RichText::new(summary).monospace().color(theme::GREEN),
                );
            }
            if let Some(error) = state.asset.figma.error.clone() {
                ui.label(
                    RichText::new(error).monospace().color(theme::RED),
                );
            }

            if ui.button("Close").clicked() {
                // Also forgets a finished run's report; a still-in-flight job
                // keeps running — closing just stops showing its progress.
                state.asset.figma.dialog_open = false;
                state.asset.figma.progress = Vec::new();
                state.asset.figma.summary = None;
                state.asset.figma.error = None;
            }
        });
}
