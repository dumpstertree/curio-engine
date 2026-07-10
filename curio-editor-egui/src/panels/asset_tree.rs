use crate::asset_state::{AssetState, TreeAction, TreeNode};
use crate::fs_ops::{self, SUPPORTED_EXTS};
use crate::theme;
use eframe::egui::{self, RichText, Ui};

fn file_icon(entry: &fs_ops::DirEntry) -> &'static str {
    if entry.is_dir {
        return "[Dir]";
    }
    match fs_ops::file_ext(&entry.name).as_str() {
        ".png" => "[Png]",
        ".glb" => "[Glb]",
        ".anim" => "[Anim]",
        ".comp" => "[Comp]",
        _ => "[File]",
    }
}

/// Renders the toolbar + tree. Selecting a supported file updates
/// `asset.selected_path`, which `asset_tab.rs` reads each frame to decide
/// what to preview — same single source of truth the original React version
/// used (`selectedEntry` drove both the tree highlight and the viewport).
pub fn show(ui: &mut Ui, asset: &mut AssetState, project_root: &str) {
    asset.ensure_roots(project_root);

    let mut actions = Vec::new();

    ui.horizontal(|ui| {
        if ui.button("Import").on_hover_text("Import file").clicked() {
            actions.push(TreeAction::Import);
        }
        if ui
            .button("New Folder")
            .on_hover_text("New folder")
            .clicked()
        {
            actions.push(TreeAction::NewFolder);
        }
        if ui
            .button("New Comp")
            .on_hover_text("New prefab (.comp)")
            .clicked()
        {
            actions.push(TreeAction::NewComp);
        }
    });
    ui.separator();

    let root_path = fs_ops::assets_root(project_root);

    // Root drop bar — shown for the whole duration of a drag, right below
    // the toolbar (so it's always on screen, not dependent on there being
    // blank space left over below a short tree, which is why dropping past
    // the last item into root never worked before: on a tree tall enough to
    // fill the panel — the common case — there was no blank space to hover
    // at all). Also doubles as live "where is this going" feedback: its
    // label tracks `drop_target` every frame, whether that's root itself or
    // whatever row is currently hovered.
    if let Some(drag_name) = asset
        .drag_path
        .as_deref()
        .map(|p| p.rsplit('/').next().unwrap_or(p).to_string())
    {
        let is_root_target = asset.drop_target.as_deref() == Some(root_path.as_str());
        let dest = match asset.drop_target.as_deref() {
            Some(t) if t == root_path => "Assets (root)".to_string(),
            Some(t) => t.rsplit('/').next().unwrap_or(t).to_string(),
            None => "…".to_string(),
        };
        let bar_resp = egui::Frame::NONE
            .fill(if is_root_target { theme::BG_HOVER } else { theme::BG_TERTIARY })
            .inner_margin(egui::Margin::symmetric(8, 6))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(RichText::new("⬆").color(theme::TEXT_MUTED));
                    ui.label(RichText::new(format!("Move \"{drag_name}\" to {dest}")).color(theme::TEXT_SECONDARY));
                });
            })
            .response;

        if bar_resp.hovered() {
            actions.push(TreeAction::DragOver(Some(root_path.clone())));
            if ui.input(|i| i.pointer.any_released()) {
                actions.push(TreeAction::Drop(root_path.clone()));
            }
        }
        ui.separator();
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
            // we need `asset` immutably (selected_path, drag state, etc.) while
            // also building the list of nodes to walk. `AssetState`'s fields
            // besides `roots` are cheap Options/Strings — clone the bits the row
            // renderer needs to compare against, rather than juggling split
            // borrows through the recursion.
            let selected_path = asset.selected_path.clone();
            let drag_path = asset.drag_path.clone();
            let drop_target = asset.drop_target.clone();
            let renaming_path = asset.renaming_path.clone();
            let confirming_delete_path = asset.confirming_delete_path.clone();

            for i in 0..asset.roots.len() {
                let node = &asset.roots[i];
                row(ui, node, 0, &selected_path, &drag_path, &drop_target, &renaming_path, &asset.rename_draft, &confirming_delete_path, &mut actions);
            }
        });

    // If the pointer is over neither the root bar nor any row this frame
    // (e.g. it's over the toolbar, or the gap between the bar and the
    // tree), clear the target rather than leaving it stuck on whatever was
    // last hovered — otherwise the bar's "Move X to ..." label could show a
    // destination the pointer isn't actually over anymore.
    if asset.drag_path.is_some() && !actions.iter().any(|a| matches!(a, TreeAction::DragOver(_))) {
        actions.push(TreeAction::DragOver(None));
    }

    // `DragEnd` (from the dragged row's own widget noticing the release)
    // and `Drop` (from whichever row/bar the pointer was actually over)
    // often both land in this frame's batch, and normally `actions` is in
    // tree-traversal order — so whichever one's row happened to be walked
    // first wins the race. If that was the dragged item itself, `DragEnd`
    // would clear `drag_path` before `Drop` got a chance to use it,
    // silently no-op'ing the move. Applying every `Drop` before any
    // `DragEnd` makes the move succeed regardless of where the source and
    // target happen to sit in the tree.
    actions.sort_by_key(|a| matches!(a, TreeAction::DragEnd));

    for action in actions {
        asset.apply(action, project_root);
    }
}

#[allow(clippy::too_many_arguments)]
fn row(ui: &mut Ui, node: &TreeNode, depth: usize, selected_path: &Option<String>, drag_path: &Option<String>, drop_target: &Option<String>, renaming_path: &Option<String>, rename_draft: &str, confirming_delete_path: &Option<String>, actions: &mut Vec<TreeAction>) {
    let entry = &node.entry;
    let ext = fs_ops::file_ext(&entry.name);
    let is_supported = entry.is_dir || SUPPORTED_EXTS.contains(&ext.as_str());
    let is_selected = selected_path.as_deref() == Some(entry.path.as_str());
    let is_dragging = drag_path.as_deref() == Some(entry.path.as_str());
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

    let row_resp = egui::Frame::NONE
        .fill(fill)
        .inner_margin(egui::Margin::symmetric(0, 1))
        .show(ui, |ui| {
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

                let icon_resp = ui.add(
                    egui::Label::new(file_icon(entry))
                        .selectable(false)
                        .sense(egui::Sense::click_and_drag()),
                );
                if icon_resp.drag_started() {
                    actions.push(TreeAction::DragStart(entry.path.clone()));
                }
                if icon_resp.drag_stopped() {
                    actions.push(TreeAction::DragEnd);
                }

                if is_renaming {
                    let mut draft = rename_draft.to_string();
                    let resp = ui.text_edit_singleline(&mut draft);
                    if draft != rename_draft {
                        // Draft changed this frame — commit isn't right here
                        // (AssetState owns `rename_draft`); we surface the new
                        // text back up as an action so `AssetState::apply` can
                        // update it. Cheaper: apply directly since AssetState is
                        // `&mut` all the way up through `asset_tab.rs`. To keep
                        // this function's signature simple we just stash it as
                        // a `StartRename` re-fire with the same path — see NOTE.
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
                            actions.push(TreeAction::SetFocusedDir(entry.path.clone()));
                        } else if is_supported {
                            actions.push(TreeAction::Select(entry.path.clone()));
                        }
                    }

                    // Drag source
                    if name_resp.drag_started() {
                        actions.push(TreeAction::DragStart(entry.path.clone()));
                    }
                    if name_resp.drag_stopped() {
                        actions.push(TreeAction::DragEnd);
                    }

                    if !entry.is_dir && !is_supported {
                        ui.label(
                            RichText::new(if ext.is_empty() { "?".to_string() } else { ext.clone() })
                                .small()
                                .color(theme::TEXT_MUTED),
                        );
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("Del").on_hover_text("Delete").clicked() {
                        actions.push(TreeAction::RequestDelete(entry.path.clone()));
                    }
                    if ui.small_button("Ren").on_hover_text("Rename").clicked() {
                        actions.push(TreeAction::StartRename(entry.path.clone(), entry.name.clone()));
                    }
                });
            });
        });

    // A folder can't be dropped into itself or one of its own descendants
    // (asset_state.rs's `Drop` handler already rejects this at the
    // filesystem-move level, but leaving these rows eligible as *hover*
    // targets meant the cursor's path from the dragged row to any real
    // target often crossed the dragged folder's own children first, and
    // those still lit up as a valid-looking target right up until release).
    let is_invalid_target = drag_path
        .as_deref()
        .is_some_and(|dp| entry.path == dp || entry.path.starts_with(&format!("{dp}/")));

    // Drop target — hovering a drag over this row while something's being
    // dragged marks it (folder) or its parent dir (file) as the target;
    // releasing over it fires the actual move.
    if drag_path.is_some() && !is_invalid_target && row_resp.response.hovered() {
        let target = if entry.is_dir { entry.path.clone() } else { entry.path[..entry.path.rfind('/').unwrap_or(0)].to_string() };
        actions.push(TreeAction::DragOver(Some(target.clone())));
        if ui.input(|i| i.pointer.any_released()) {
            actions.push(TreeAction::Drop(target));
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
            row(ui, child, depth + 1, selected_path, drag_path, drop_target, renaming_path, rename_draft, confirming_delete_path, actions);
        }
    }
}
