use crate::state::{EditorState, TopTab};
use crate::theme;
use eframe::egui::{self, RichText, Ui};

const TABS: &[(TopTab, &str, bool)] = &[
    (TopTab::Play, "\u{25B6} Play", false),
    (TopTab::Asset, "Asset", false),
    (TopTab::Input, "Input", true),
    (TopTab::Prefab, "Prefab", true),
];

pub fn show(ui: &mut Ui, state: &mut EditorState) {
    ui.horizontal(|ui| {
        for &(tab, label, tbd) in TABS {
            let active = state.active_tab == tab;

            let text = if active {
                RichText::new(label).color(theme::TEXT_PRIMARY).strong()
            } else {
                RichText::new(label).color(theme::TEXT_SECONDARY)
            };

            let button = egui::Button::new(text).fill(if active { theme::BG_TERTIARY } else { egui::Color32::TRANSPARENT }).stroke(egui::Stroke::NONE);

            let resp = ui.add(button);
            if tbd {
                ui.label(RichText::new("TBD").small().color(theme::TEXT_MUTED));
            }
            if resp.clicked() {
                state.active_tab = tab;
            }
        }
    });
}
