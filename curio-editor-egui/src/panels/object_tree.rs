use crate::state::ObjectPath;
use crate::theme;
use curio_core::ObjectState;
use eframe::egui::{self, RichText, Ui};
use std::collections::HashSet;

/// Renders the object tree for one tab's root objects.
/// Takes ownership of nothing — `objects` is expected to already be a
/// detached clone of the current snapshot (see `left_panel::show`), so this
/// can freely mutate `selected_path`/`expanded` without borrow conflicts
/// against the snapshot it's rendering.
pub fn show(ui: &mut Ui, objects: &[ObjectState], selected_path: &mut Option<ObjectPath>, expanded: &mut HashSet<String>) {
    if objects.is_empty() {
        ui.weak("No objects");
        return;
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (i, obj) in objects.iter().enumerate() {
                node_row(ui, obj, &format!("root/{}{}", obj.object_name, i), 0, &[i], selected_path, expanded);
            }
        });
}

#[allow(clippy::too_many_arguments)]
fn node_row(ui: &mut Ui, obj: &ObjectState, path: &str, depth: usize, index_path: &[usize], selected_path: &mut Option<ObjectPath>, expanded: &mut HashSet<String>) {
    let has_children = !obj.children.is_empty();
    let is_selected = selected_path.as_deref() == Some(index_path);
    let is_expanded = expanded.contains(path);

    let fill = if is_selected { theme::BG_ACTIVE } else { egui::Color32::TRANSPARENT };

    egui::Frame::NONE
        .fill(fill)
        .inner_margin(egui::Margin::symmetric(0, 1))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(depth as f32 * 16.0 + 2.0);

                // Chevron
                if has_children {
                    if crate::icons::chevron(ui, is_expanded, theme::TEXT_SECONDARY).clicked() {
                        if is_expanded {
                            expanded.remove(path);
                        } else {
                            expanded.insert(path.to_string());
                        }
                    }
                } else {
                    ui.add_space(10.0);
                }

                // Node icon: hollow circle = has children, filled = leaf
                crate::icons::dot(ui, !has_children, 3.0, theme::TEXT_MUTED);

                // Name (click to select/deselect)
                let name_color = if is_selected { theme::BLUE } else { theme::TEXT_PRIMARY };
                let name_resp = ui.add(egui::Label::new(RichText::new(&obj.object_name).color(name_color)).sense(egui::Sense::click()));
                if name_resp.clicked() {
                    *selected_path = if is_selected { None } else { Some(index_path.to_vec()) };
                }

                if !obj.components.is_empty() {
                    ui.label(
                        RichText::new(obj.components.len().to_string())
                            .small()
                            .color(theme::TEXT_MUTED),
                    );
                }
            });
        });

    if is_expanded {
        for (i, child) in obj.children.iter().enumerate() {
            let mut child_index_path = index_path.to_vec();
            child_index_path.push(i);
            let child_path = format!("{path}/{}{i}", child.object_name);
            node_row(ui, child, &child_path, depth + 1, &child_index_path, selected_path, expanded);
        }
    }
}
