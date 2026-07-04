use crate::state::EditorState;
use crate::theme;
use curio_core::ComponentState;
use eframe::egui::{self, CollapsingHeader, RichText, Ui};

pub fn show(ui: &mut Ui, state: &EditorState) {
    ui.vertical(|ui| {
        ui.label(RichText::new("Inspector").strong().color(theme::TEXT_PRIMARY));
        ui.separator();

        let Some(obj) = state.selected_object() else {
            ui.weak("Select an object");
            return;
        };

        ui.label(RichText::new(&obj.object_name).strong());
        let mut meta = format!("{} component{}", obj.components.len(), if obj.components.len() == 1 { "" } else { "s" });
        if !obj.children.is_empty() {
            meta.push_str(&format!(" · {} children", obj.children.len()));
        }
        ui.label(RichText::new(meta).small().color(theme::TEXT_SECONDARY));
        ui.add_space(6.0);

        if obj.components.is_empty() {
            ui.weak("No components");
            return;
        }

        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            for (i, comp) in obj.components.iter().enumerate() {
                component_block(ui, comp, i);
            }
        });
    });
}

fn component_block(ui: &mut Ui, comp: &ComponentState, idx: usize) {
    CollapsingHeader::new(RichText::new(&comp.component_name).color(theme::GREEN)).id_salt(("comp", idx)).default_open(true).show(ui, |ui| {
        if comp.fields.is_empty() {
            ui.weak("no fields");
            return;
        }
        egui::Grid::new(("comp_fields", idx)).num_columns(2).spacing([12.0, 3.0]).striped(false).show(ui, |ui| {
            for field in &comp.fields {
                ui.label(RichText::new(&field.field_name).color(theme::TEXT_SECONDARY));
                // NOTE: `field.data`'s exact shape depends on curio_core's
                // FieldState definition, which wasn't available at rewrite
                // time — rendered via `Debug` here rather than the
                // type-specific coloring the original TS inspector did
                // (string/number/bool/object/array each got their own
                // style). Swap this for a `match` over the real enum once
                // that type is visible.
                ui.label(RichText::new(format!("{:?}", field.data)).monospace().color(theme::BLUE));
                ui.end_row();
            }
        });
    });
}
