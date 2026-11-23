use crate::asset_state::{AssetState, TreeAction, TreeNode};
use crate::fs_ops::{self, SUPPORTED_EXTS};
use crate::theme;
use eframe::egui::{self, RichText, Ui};

fn file_icon_kind(entry: &fs_ops::DirEntry) -> crate::icons::FileIconKind {
    use crate::icons::FileIconKind;
    if entry.is_dir {
        return FileIconKind::Dir;
    }
    match fs_ops::file_ext(&entry.name).as_str() {
        ".png" => FileIconKind::Png,
        ".glb" => FileIconKind::Glb,
        ".anim" => FileIconKind::Anim,
        ".comp" => FileIconKind::Comp,
        _ => FileIconKind::Other,
    }
}

/// Renders the toolbar + tree. Selecting a supported file updates
/// `asset.selected_path`, which `asset_tab.rs` reads each frame to decide
/// what to preview — same single source of truth the original React version
/// used (`selectedEntry` drove both the tree highlight and the viewport).
///
/// Drag-and-drop uses egui's built-in `DragAndDrop` plugin/`dnd_*` `Response`
/// methods rather than hand-rolled `Sense::drag()` + `.hovered()` tracking.
/// That first attempt looked reasonable but was fundamentally broken:
/// `Response::hovered()` is *always false* for a widget other than the one
/// currently owning an active drag (see egui's own doc comment on
/// `hovered()`), so a target row's hover check could never actually fire
/// while genuinely dragging from a different row — the only thing that
/// "worked" was dragging an item onto itself, because that's the one case
/// where the widget being checked *is* the one owning the drag. The `dnd_*`
/// methods use `Response::contains_pointer()` instead, which — per its own
/// doc comment — stays accurate for other widgets specifically so it can be
/// used for drag-and-drop targets.
pub fn show(ui: &mut Ui, asset: &mut AssetState, project_root: &str) {
    asset.ensure_roots(project_root);
    asset.poll_import(project_root);

    let mut actions = Vec::new();
    // The dragged item's path, if any — this is now owned entirely by
    // egui's `DragAndDrop` plugin (set via `dnd_set_drag_payload` below)
    // rather than tracked by hand; reading it here is always this frame's
    // live value, no staleness to worry about.
    let dragging: Option<String> = egui::DragAndDrop::payload::<String>(ui.ctx()).map(|s| (*s).clone());

    let root_path = fs_ops::assets_root(project_root);
    let is_dragging = dragging.is_some();

    // Top root drop bar — always on screen (not dependent on leftover blank
    // space below the tree, which a tall list — the common case — doesn't
    // have) so root is reachable without having to scroll.
    if is_dragging {
        root_drop_bar(ui, &root_path, &mut actions);
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if let Some(err) = &asset.load_error {
                ui.colored_label(theme::RED, err);
                return;
            }
            if asset.roots.is_empty() {
                ui.weak("Empty folder");
                return;
            }

            // Two-pass: collect a read-only snapshot of what's rendered, since
            // we need `asset` immutably (selected_path, drop state, etc.) while
            // also building the list of nodes to walk. `AssetState`'s fields
            // besides `roots` are cheap Options/Strings — clone the bits the row
            // renderer needs to compare against, rather than juggling split
            // borrows through the recursion.
            let selected_path = asset.selected_path.clone();
            let drop_target = asset.drop_target.clone();
            let renaming_path = asset.renaming_path.clone();
            let confirming_delete_path = asset.confirming_delete_path.clone();

            for i in 0..asset.roots.len() {
                let node = &asset.roots[i];
                row(ui, node, 0, &selected_path, &dragging, &drop_target, &renaming_path, &asset.rename_draft, &confirming_delete_path, &mut actions);
            }

            // Bottom root drop bar — same target as the top one, but reachable
            // by dragging *past* the last item instead of back up to the top.
            // It's part of the scrollable content (not pinned below it), so
            // it's still reachable by scrolling down through a long list.
            if is_dragging {
                root_drop_bar(ui, &root_path, &mut actions);
            }
        });

    // If the pointer is over neither a drop bar nor any row this frame
    // (e.g. it's over the toolbar), clear the target rather than leaving
    // it stuck on whatever was last hovered — it only drives the row
    // highlight now, but a stale value there would still look wrong.
    if dragging.is_some() && !actions.iter().any(|a| matches!(a, TreeAction::DragOver(_))) {
        actions.push(TreeAction::DragOver(None));
    }

    for action in actions {
        asset.apply(action, project_root);
    }
}

/// Right-click-to-create-at-root, hung off a `Response` the caller already
/// has (asset_tab.rs's own "Assets" panel title) rather than asset_tree.rs
/// rendering a second "Assets" label of its own just to have something to
/// attach a context menu to.
pub fn attach_root_context_menu(response: egui::Response, asset: &mut AssetState, project_root: &str) {
    let root_path = fs_ops::assets_root(project_root);
    let mut actions = Vec::new();
    response.context_menu(|ui| creation_menu(ui, &root_path, &mut actions));
    for action in actions {
        asset.apply(action, project_root);
    }
}

/// The Import/New Folder/New Comp menu, shared between the asset panel's
/// title (see `attach_root_context_menu` — targets root) and each row's
/// right-click (targets the row itself if it's a folder, or its parent dir
/// if it's a file — a "sibling" of whatever was right-clicked).
pub(crate) fn creation_menu(ui: &mut Ui, target_dir: &str, actions: &mut Vec<TreeAction>) {
    if ui.button("Import...").clicked() {
        actions.push(TreeAction::Import(target_dir.to_string()));
        ui.close_menu();
    }
    if ui.button("New Folder").clicked() {
        actions.push(TreeAction::NewFolder(target_dir.to_string()));
        ui.close_menu();
    }
    if ui.button("New Comp").clicked() {
        actions.push(TreeAction::NewComp(target_dir.to_string()));
        ui.close_menu();
    }
}

/// A row's right-click menu: `creation_menu` (targeting this row — see its
/// doc comment) plus Rename/Delete for the row itself, mirroring the
/// hover-revealed "Ren"/"Del" buttons (see `row()`) as a second way to
/// reach the same actions.
fn row_context_menu(ui: &mut Ui, entry: &fs_ops::DirEntry, target_dir: &str, actions: &mut Vec<TreeAction>) {
    creation_menu(ui, target_dir, actions);
    ui.separator();
    if ui.button("Rename").clicked() {
        actions.push(TreeAction::StartRename(entry.path.clone(), entry.name.clone()));
        ui.close_menu();
    }
    if ui.button("Delete").clicked() {
        actions.push(TreeAction::RequestDelete(entry.path.clone()));
        ui.close_menu();
    }
}

/// A thin horizontal line, live-highlighted while a drag is hovering it,
/// that drops the dragged item at `root_path`. Used both above and below
/// the tree so root is reachable by dragging to either end of the list,
/// not just the top.
fn root_drop_bar(ui: &mut Ui, root_path: &str, actions: &mut Vec<TreeAction>) {
    let size = egui::vec2(ui.available_width(), 8.0);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::hover());
    let hovered = resp.dnd_hover_payload::<String>().is_some();

    let line_height = if hovered { 3.0 } else { 2.0 };
    let color = if hovered { theme::BLUE } else { theme::TEXT_MUTED.gamma_multiply(0.5) };
    let line_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(rect.width(), line_height));
    ui.painter().rect_filled(line_rect, 1.0, color);

    if hovered {
        actions.push(TreeAction::DragOver(Some(root_path.to_string())));
    }
    if let Some(source) = resp.dnd_release_payload::<String>() {
        actions.push(TreeAction::Drop {
            source: (*source).clone(),
            target: root_path.to_string(),
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn row(ui: &mut Ui, node: &TreeNode, depth: usize, selected_path: &Option<String>, dragging: &Option<String>, drop_target: &Option<String>, renaming_path: &Option<String>, rename_draft: &str, confirming_delete_path: &Option<String>, actions: &mut Vec<TreeAction>) {
    let entry = &node.entry;
    let ext = fs_ops::file_ext(&entry.name);
    let is_supported = entry.is_dir || SUPPORTED_EXTS.contains(&ext.as_str());
    let is_selected = selected_path.as_deref() == Some(entry.path.as_str());
    let is_dragging = dragging.as_deref() == Some(entry.path.as_str());
    let is_drop_target = entry.is_dir && drop_target.as_deref() == Some(entry.path.as_str());
    let is_renaming = renaming_path.as_deref() == Some(entry.path.as_str());

    if !entry.is_dir && node.meta.is_none() {
        actions.push(TreeAction::EnsureMeta(entry.path.clone()));
    }

    let fill = if is_selected {
        theme::BG_ACTIVE
    } else if is_drop_target {
        theme::BG_HOVER
    } else if is_dragging {
        theme::BG_TERTIARY
    } else {
        egui::Color32::TRANSPARENT
    };

    // Where a right-click-created/imported item, or a dropped item, lands
    // relative to this row: a folder's own path (i.e. as its child), or a
    // file's parent dir (i.e. as its sibling).
    let target_dir = if entry.is_dir { entry.path.clone() } else { entry.path[..entry.path.rfind('/').unwrap_or(0)].to_string() };

    let row_resp = egui::Frame::NONE
        .fill(fill)
        .inner_margin(egui::Margin::symmetric(0, 1))
        .show(ui, |ui| {
            // Without this, the frame only grows as wide as its content
            // (chevron/icon/name), so `allocate_ui_at_rect` below — which
            // right-anchors the hover Del/Ren buttons to *this* rect —ends
            // up anchoring them right against the end of the name text
            // instead of the panel's actual right edge.
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.add_space(depth as f32 * 14.0 + 4.0);

                // Chevron
                if entry.is_dir {
                    if crate::icons::chevron(ui, node.expanded, theme::TEXT_SECONDARY).clicked() {
                        actions.push(TreeAction::ToggleExpand(entry.path.clone()));
                    }
                } else {
                    ui.add_space(10.0);
                }

                // Include checkbox (files only)
                if !entry.is_dir {
                    let mut included = node.meta.as_ref().map(|m| m.included).unwrap_or(true);
                    let resp = ui.checkbox(&mut included, "");
                    if resp.changed() {
                        actions.push(TreeAction::ToggleIncluded(entry.path.clone()));
                    }
                    if let Some(meta) = &node.meta {
                        resp.on_hover_text(format!("{} (ID: {})", if meta.included { "Included" } else { "Excluded" }, meta.id));
                    }
                }

                let icon_color = if is_selected {
                    theme::BLUE
                } else if is_supported {
                    theme::TEXT_SECONDARY
                } else {
                    theme::TEXT_MUTED
                };
                let icon_resp = crate::icons::file_icon(ui, file_icon_kind(entry), icon_color);
                icon_resp.dnd_set_drag_payload(entry.path.clone());
                icon_resp.context_menu(|ui| {
                    row_context_menu(ui, entry, &target_dir, &mut *actions);
                });

                if is_renaming {
                    let mut draft = rename_draft.to_string();
                    let resp = ui.text_edit_singleline(&mut draft);
                    if draft != rename_draft {
                        // Every keystroke re-fires StartRename with the
                        // updated text so AssetState::apply's StartRename
                        // handler (which sets both renaming_path and
                        // rename_draft) is the single place that updates it.
                        actions.push(TreeAction::StartRename(entry.path.clone(), draft));
                    }
                    if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        actions.push(TreeAction::CommitRename(entry.path.clone()));
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        actions.push(TreeAction::CancelRename);
                    }
                    resp.request_focus();
                } else {
                    let name_color = if is_selected {
                        theme::BLUE
                    } else if is_supported {
                        theme::TEXT_PRIMARY
                    } else {
                        theme::TEXT_MUTED
                    };
                    let name_resp = ui.add(
                        egui::Label::new(RichText::new(&entry.name).color(name_color))
                            .selectable(false)
                            .sense(egui::Sense::click_and_drag()),
                    );

                    if name_resp.clicked() {
                        if entry.is_dir {
                            actions.push(TreeAction::ToggleExpand(entry.path.clone()));
                        } else if is_supported {
                            actions.push(TreeAction::Select(entry.path.clone()));
                        }
                    }

                    // Drag source — see `show()`'s doc comment for why this
                    // (and `icon_resp` above) use egui's `dnd_*` helpers
                    // instead of hand-tracked drag state.
                    name_resp.dnd_set_drag_payload(entry.path.clone());
                    name_resp.context_menu(|ui| {
                        row_context_menu(ui, entry, &target_dir, &mut *actions);
                    });

                    if !entry.is_dir && !is_supported {
                        ui.label(
                            RichText::new(if ext.is_empty() { "?".to_string() } else { ext.clone() })
                                .small()
                                .color(theme::TEXT_MUTED),
                        );
                    }
                }
            });
        });

    // Del/Ren buttons — only drawn when the row is actually hovered, rather
    // than reserving permanent space in the row's own layout. Has to be a
    // separate overlay positioned at the row's now-known rect rather than
    // part of the row's own content, since whether to show them at all
    // depends on `row_resp`, which only exists *after* the row is drawn.
    //
    // Uses `contains_pointer()` rather than `hovered()`: the buttons below
    // are a second, separate widget layered on top of this same rect, so
    // the instant the pointer moved onto *them* to click one, they'd become
    // the topmost thing there — and `hovered()` (unlike `contains_pointer()`)
    // excludes points some other widget is covering, so the row would stop
    // reporting itself hovered and the buttons would vanish right as you
    // tried to click them.
    if row_resp.response.contains_pointer() && !is_renaming {
        ui.allocate_ui_at_rect(row_resp.response.rect, |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(4.0);
                if ui.small_button("Del").on_hover_text("Delete").clicked() {
                    actions.push(TreeAction::RequestDelete(entry.path.clone()));
                }
                if ui.small_button("Ren").on_hover_text("Rename").clicked() {
                    actions.push(TreeAction::StartRename(entry.path.clone(), entry.name.clone()));
                }
            });
        });
    }

    // A folder can't be dropped into itself or one of its own descendants
    // (asset_state.rs's `Drop` handler already rejects this at the
    // filesystem-move level, but leaving these rows eligible as drop
    // targets meant the cursor's path from the dragged row to any real
    // target often crossed the dragged folder's own children first, and
    // those still lit up as a valid-looking target right up until release).
    let is_invalid_target = dragging
        .as_deref()
        .is_some_and(|dp| entry.path == dp || entry.path.starts_with(&format!("{dp}/")));

    // Drop target — `dnd_hover_payload`/`dnd_release_payload` check
    // `contains_pointer()` internally, which (unlike `.hovered()`) is still
    // accurate for this row while a *different* row's widget owns the
    // active drag. That's the actual fix here; see `show()`'s doc comment.
    if !is_invalid_target {
        if row_resp.response.dnd_hover_payload::<String>().is_some() {
            actions.push(TreeAction::DragOver(Some(target_dir.clone())));
        }
        if let Some(source) = row_resp.response.dnd_release_payload::<String>() {
            actions.push(TreeAction::Drop { source: (*source).clone(), target: target_dir });
        }
    }

    if let Some(confirm_path) = confirming_delete_path {
        if confirm_path == &entry.path {
            ui.horizontal(|ui| {
                ui.add_space(depth as f32 * 14.0 + 20.0);
                let msg = if entry.is_dir { "Delete folder + all contents?".to_string() } else { format!("Delete \"{}\"?", entry.name) };
                ui.colored_label(theme::RED, msg);
                if ui.small_button("Yes").clicked() {
                    actions.push(TreeAction::ConfirmDelete(entry.path.clone()));
                }
                if ui.small_button("No").clicked() {
                    actions.push(TreeAction::CancelDelete);
                }
            });
        }
    }

    if entry.is_dir && node.expanded {
        for child in &node.children {
            row(ui, child, depth + 1, selected_path, dragging, drop_target, renaming_path, rename_draft, confirming_delete_path, actions);
        }
    }
}
