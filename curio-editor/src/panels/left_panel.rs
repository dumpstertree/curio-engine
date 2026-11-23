use crate::panels::object_tree;
use crate::state::EditorState;
use crate::theme;
use eframe::egui::{self, RichText, Ui};

pub fn show(ui: &mut Ui, state: &mut EditorState) {
    ui.vertical(|ui| {
        ui.set_width(ui.available_width());

        // ── Instance dropdown ───────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label(RichText::new("Instance").small().color(theme::TEXT_SECONDARY));

            let mut keys: Vec<String> = state.tab_group_state.as_ref().map(|t| t.id_for_tabs.keys().cloned().collect()).unwrap_or_default();
            keys.sort();

            let selected_label = if state.selected_instance.is_empty() { "No instances".to_string() } else { state.selected_instance.clone() };

            egui::ComboBox::from_id_salt("instance_dropdown").selected_text(selected_label).width(ui.available_width()).show_ui(ui, |ui| {
                for key in &keys {
                    if ui.selectable_label(*key == state.selected_instance, key).clicked() {
                        state.select_instance(key.clone());
                    }
                }
            });
        });

        ui.add_space(4.0);

        // ── Dynamic tab strip ───────────────────────────────────────────────
        let tab_names: Vec<String> = state
            .tab_group_state
            .as_ref()
            .and_then(|t| t.id_for_tabs.get(&state.selected_instance))
            .map(|tabs| tabs.iter().map(|t| t.tab_name.clone()).collect())
            .unwrap_or_default();

        if tab_names.is_empty() {
            ui.weak("No tabs");
        } else {
            ui.horizontal_wrapped(|ui| {
                for (idx, name) in tab_names.iter().enumerate() {
                    let active = state.active_left_tab == idx;
                    let text = if active { RichText::new(name).color(theme::TEXT_PRIMARY) } else { RichText::new(name).color(theme::TEXT_SECONDARY) };
                    let btn = egui::Button::new(text).fill(if active { theme::BG_TERTIARY } else { egui::Color32::TRANSPARENT }).stroke(egui::Stroke::NONE);
                    if ui.add(btn).clicked() {
                        state.set_active_left_tab(idx);
                    }
                }
            });
        }

        ui.separator();

        // ── Object tree ──────────────────────────────────────────────────────
        if state.tab_group_state.is_none() {
            ui.weak("No data");
        } else {
            // Detach a clone of the active tab's objects so `object_tree::show`
            // can mutate `state.selected_object_path` / `state.expanded_nodes`
            // without fighting the borrow checker over `state.tab_group_state`.
            let objects = state
                .tab_group_state
                .as_ref()
                .and_then(|t| t.id_for_tabs.get(&state.selected_instance))
                .and_then(|tabs| tabs.get(state.active_left_tab))
                .map(|tab| tab.objects.clone())
                .unwrap_or_default();

            object_tree::show(ui, &objects, &mut state.selected_object_path, &mut state.expanded_nodes);
        }
    });
}
