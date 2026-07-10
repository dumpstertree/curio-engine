//! Port of `AssetTab.tsx`'s layout. The file tree and the include/rename/
//! delete/import/drag-drop machinery around it are fully working (see
//! `asset_tree.rs`/`asset_state.rs`). PNG, GLB, `.anim` (Spine), and now
//! `.comp` (prefab composition) all have real previews/editors
//! (`png_viewer.rs`/`glb_viewer.rs`/`anim_viewer.rs`/`panels/prefab_tab.rs`).
//! Nothing left unported at the asset-type level — see `prefab_viewer.rs`'s
//! doc comment for the prefab-specific scope cuts (no transform gizmos, no
//! real Spine-in-3D rendering).

use crate::fs_ops;
use crate::state::EditorState;
use crate::theme;
use eframe::egui::{self, RichText, Ui};

pub fn show_tree(ui: &mut Ui, state: &mut EditorState) {
    ui.vertical(|ui| {
        theme::section_title(ui, "Assets");
        ui.separator();
        let project_path = state.project_path.clone();
        crate::panels::asset_tree::show(ui, &mut state.asset, &project_path);
    });
}

pub fn show_viewport(ui: &mut Ui, state: &mut EditorState) {
    let Some(path) = state.asset.selected_path.clone() else {
        ui.centered_and_justified(|ui| {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("No selection").color(theme::TEXT_MUTED));
                ui.add_space(6.0);
                ui.label(RichText::new("Select an asset to preview").color(theme::TEXT_MUTED));
            });
        });
        return;
    };

    let name = path.rsplit('/').next().unwrap_or(&path).to_string();
    let ext = fs_ops::file_ext(&name);

    match ext.as_str() {
        ".png" => png_viewport(ui, state, &path),
        ".glb" => glb_viewport(ui, state, &path),
        ".anim" => anim_viewport(ui, state, &path),
        ".comp" => crate::panels::prefab_tab::show_viewport(ui, state, &path),
        _ => unsupported_placeholder(ui, &name),
    }
}

/// Called from `app.rs`'s right-side panel routing — `.comp` gets the
/// prefab tree/component inspector instead of the generic meta-info panel
/// every other asset type uses.
pub fn show_inspector_for_selected(ui: &mut Ui, state: &mut EditorState) {
    let Some(path) = state.asset.selected_path.clone() else {
        show_inspector(ui, state);
        return;
    };
    let name = path.rsplit('/').next().unwrap_or(&path);
    if fs_ops::file_ext(name) == ".comp" {
        crate::panels::prefab_tab::show_inspector(ui, state, &path);
    } else {
        show_inspector(ui, state);
    }
}

// ── PNG ──────────────────────────────────────────────────────────────────────

fn png_viewport(ui: &mut Ui, state: &mut EditorState, path: &str) {
    let ctx = ui.ctx().clone();
    state.ensure_png_preview(&ctx, path);

    if let Some(err) = &state.png_preview_error {
        error_box(ui, "PNG", err);
        return;
    }
    let Some(preview) = &state.png_preview else {
        ui.centered_and_justified(|ui| ui.label(RichText::new("Loading...").color(theme::TEXT_MUTED)));
        return;
    };

    ui.vertical(|ui| {
        let avail = ui.available_size();
        let tex_aspect = preview.width as f32 / preview.height.max(1) as f32;
        let avail_aspect = avail.x / avail.y.max(1.0);
        let size = if avail_aspect > tex_aspect { egui::vec2(avail.y * tex_aspect, avail.y) } else { egui::vec2(avail.x, avail.x / tex_aspect) };

        ui.centered_and_justified(|ui| {
            ui.add(egui::Image::new(&preview.texture).fit_to_exact_size(size));
        });
    });
}

// ── GLB ──────────────────────────────────────────────────────────────────────

fn glb_viewport(ui: &mut Ui, state: &mut EditorState, path: &str) {
    state.ensure_glb_preview(path);

    if let Some(err) = &state.glb_preview_error {
        error_box(ui, "GLB", err);
        return;
    }

    let Some(render_shared) = state.render_shared().cloned() else {
        ui.centered_and_justified(|ui| ui.label(RichText::new("No GPU device available").color(theme::RED)));
        return;
    };

    let Some(preview) = &mut state.glb_preview else {
        ui.centered_and_justified(|ui| ui.label(RichText::new("Loading...").color(theme::TEXT_MUTED)));
        return;
    };

    ui.vertical(|ui| {
        // Top bar is actions only (mesh/triangle counts moved to the
        // inspector panel — see `show_inspector`'s `.glb` branch).
        ui.horizontal(|ui| {
            if ui.small_button("Reset camera").clicked() {
                preview.reset_camera();
            }
        });

        let avail = ui.available_size();
        let width = avail.x.max(1.0) as u32;
        let height = avail.y.max(1.0) as u32;

        let (texture_id, size) = preview.render(&render_shared.render_state, width, height);

        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
        ui.painter()
            .image(texture_id, rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);

        if response.dragged() {
            preview.handle_input(response.drag_delta(), 0.0);
        }
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if response.hovered() && scroll != 0.0 {
            preview.handle_input(egui::Vec2::ZERO, scroll);
        }

        // Orbiting needs continuous repaint while the mouse is down/hovered
        // with scroll input, same as the game viewport does while playing.
        if response.dragged() || (response.hovered() && scroll != 0.0) {
            ui.ctx().request_repaint();
        }
    });
}

// ── Spine (.anim) ────────────────────────────────────────────────────────────

fn anim_viewport(ui: &mut Ui, state: &mut EditorState, path: &str) {
    state.ensure_anim_preview(path);

    if let Some(err) = &state.anim_preview_error {
        error_box(ui, "Spine (.anim)", err);
        return;
    }

    let Some(render_shared) = state.render_shared().cloned() else {
        ui.centered_and_justified(|ui| ui.label(RichText::new("No GPU device available").color(theme::RED)));
        return;
    };

    let Some(preview) = &mut state.anim_preview else {
        ui.centered_and_justified(|ui| ui.label(RichText::new("Loading...").color(theme::TEXT_MUTED)));
        return;
    };

    ui.vertical(|ui| {
        // Top bar is actions only (bone/slot counts moved to the inspector
        // panel — see `show_inspector`'s `.anim` branch). The animation
        // picker and elapsed/duration readout stay here since they're
        // playback controls/live status, not static asset info.
        ui.horizontal(|ui| {
            let current = preview.current_animation.clone();
            egui::ComboBox::from_id_salt("anim_selector")
                .selected_text(if current.is_empty() { "(none)".to_string() } else { current.clone() })
                .show_ui(ui, |ui| {
                    for name in preview.animations.clone() {
                        if ui.selectable_label(name == current, &name).clicked() {
                            preview.set_animation(&name);
                        }
                    }
                });

            ui.separator();
            ui.label(
                RichText::new(format!("{:.2}s / {:.2}s", preview.elapsed, preview.duration))
                    .small()
                    .monospace()
                    .color(theme::TEXT_MUTED),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("Reset camera").clicked() {
                    preview.reset_camera();
                }
            });
        });

        let avail = ui.available_size();
        let width = avail.x.max(1.0) as u32;
        let height = avail.y.max(1.0) as u32;
        let aspect = width as f32 / height.max(1) as f32;

        let (texture_id, size) = preview.render(&render_shared.render_state, width, height);

        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
        ui.painter()
            .image(texture_id, rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);

        if response.dragged() {
            preview.handle_input(response.drag_delta(), 0.0, aspect);
        }
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if response.hovered() && scroll != 0.0 {
            preview.handle_input(egui::Vec2::ZERO, scroll, aspect);
        }

        // Animation playback needs continuous repaint regardless of input —
        // unlike GLB (a static mesh, repaints only while orbiting), this is
        // always advancing.
        ui.ctx().request_repaint();
    });
}

// ── Stubs ────────────────────────────────────────────────────────────────────

fn error_box(ui: &mut Ui, kind: &str, message: &str) {
    ui.centered_and_justified(|ui| {
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(format!("{kind} preview failed")).color(theme::RED));
            ui.label(RichText::new(message).small().color(theme::TEXT_MUTED));
        });
    });
}

fn unsupported_placeholder(ui: &mut Ui, name: &str) {
    ui.centered_and_justified(|ui| {
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("No preview available").color(theme::TEXT_SECONDARY));
            ui.label(RichText::new(name).monospace().color(theme::BLUE));
        });
    });
}

/// Takes `&mut EditorState` (not just `&EditorState`) specifically so the
/// `.glb`/`.anim` branches below can call `ensure_glb_preview`/
/// `ensure_anim_preview` themselves — the inspector panel renders *before*
/// the viewport within a frame (see `app.rs`'s panel ordering), so without
/// this the mesh/bone info shown here would lag one frame behind on first
/// load. Both `ensure_*` calls are cheap no-ops if the right asset is
/// already loaded (which it will be, every frame after the first).
pub fn show_inspector(ui: &mut Ui, state: &mut EditorState) {
    ui.vertical(|ui| {
        theme::section_title(ui, "Inspector");
        ui.separator();

        let Some(path) = state.asset.selected_path.clone() else {
            ui.weak("No asset selected");
            return;
        };

        let name = path.rsplit('/').next().unwrap_or(&path).to_string();
        theme::strong_label(ui, &name, theme::TEXT_PRIMARY);
        ui.label(
            RichText::new(path.as_str())
                .small()
                .color(theme::TEXT_MUTED),
        );
        ui.add_space(6.0);

        // Meta info, if it's been loaded by the tree yet — mirrors what
        // AssetInspectorView.tsx showed above the type-specific fields.
        if let Some(meta) = find_meta(&state.asset.roots, &path) {
            egui::Grid::new("asset_meta_grid")
                .num_columns(2)
                .spacing([12.0, 3.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("ID").color(theme::TEXT_SECONDARY));
                    ui.label(RichText::new(meta.id.to_string()).monospace());
                    ui.end_row();
                    ui.label(RichText::new("Included").color(theme::TEXT_SECONDARY));
                    ui.label(RichText::new(meta.included.to_string()).monospace());
                    ui.end_row();
                });
            ui.add_space(6.0);
        }

        let ext = fs_ops::file_ext(&name);
        match ext.as_str() {
            ".png" => {
                if let Some(preview) = &state.png_preview {
                    if preview.path == path {
                        ui.separator();
                        theme::strong_label(ui, "Image", theme::TEXT_SECONDARY);
                        egui::Grid::new("png_info_grid")
                            .num_columns(2)
                            .spacing([12.0, 3.0])
                            .show(ui, |ui| {
                                ui.label(RichText::new("Dimensions").color(theme::TEXT_SECONDARY));
                                ui.label(RichText::new(format!("{} x {} px", preview.width, preview.height)).monospace());
                                ui.end_row();
                            });
                    }
                }
            }
            ".glb" => {
                state.ensure_glb_preview(&path);
                if let Some(preview) = &state.glb_preview {
                    if preview.path == path {
                        ui.separator();
                        theme::strong_label(ui, "Mesh", theme::TEXT_SECONDARY);
                        egui::Grid::new("glb_info_grid")
                            .num_columns(2)
                            .spacing([12.0, 3.0])
                            .show(ui, |ui| {
                                ui.label(RichText::new("Meshes").color(theme::TEXT_SECONDARY));
                                ui.label(RichText::new(preview.mesh_count.to_string()).monospace());
                                ui.end_row();
                                ui.label(RichText::new("Triangles").color(theme::TEXT_SECONDARY));
                                ui.label(RichText::new(preview.triangle_count.to_string()).monospace());
                                ui.end_row();
                            });
                    }
                }
            }
            ".anim" => {
                state.ensure_anim_preview(&path);
                if let Some(preview) = &state.anim_preview {
                    if preview.path == path {
                        ui.separator();
                        theme::strong_label(ui, "Skeleton", theme::TEXT_SECONDARY);
                        egui::Grid::new("anim_info_grid")
                            .num_columns(2)
                            .spacing([12.0, 3.0])
                            .show(ui, |ui| {
                                ui.label(RichText::new("Bones").color(theme::TEXT_SECONDARY));
                                ui.label(RichText::new(preview.bone_count.to_string()).monospace());
                                ui.end_row();
                                ui.label(RichText::new("Slots").color(theme::TEXT_SECONDARY));
                                ui.label(RichText::new(preview.slot_count.to_string()).monospace());
                                ui.end_row();
                                ui.label(RichText::new("Animations").color(theme::TEXT_SECONDARY));
                                ui.label(RichText::new(preview.animations.len().to_string()).monospace());
                                ui.end_row();
                            });
                    }
                }
            }
            _ => {}
        }
    });
}

fn find_meta<'a>(nodes: &'a [crate::asset_state::TreeNode], path: &str) -> Option<&'a fs_ops::MetaFile> {
    for node in nodes {
        if node.entry.path == path {
            return node.meta.as_ref();
        }
        if let Some(found) = find_meta(&node.children, path) {
            return Some(found);
        }
    }
    None
}
