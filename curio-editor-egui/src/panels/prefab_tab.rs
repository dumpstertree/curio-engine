//! Port of `PrefabViewport.tsx` — including the move/rotate/scale gizmo
//! (`prefab_gizmo.rs`; earlier passes shipped this viewport without it) —
//! and `PrefabInspectorView.tsx` (tree + component/field editing).

use crate::fs_ops;
use crate::prefab_gizmo::{self, GizmoTarget};
use crate::prefab_resolver;
use crate::prefab_state::{GizmoMode, PrefabAction};
use crate::prefab_transforms;
use crate::prefab_types::{self, EntryType, FieldDescriptor, PrefabComponentRaw, PrefabGameObjectRaw};
use crate::state::EditorState;
use crate::theme;
use eframe::egui::{self, RichText, Ui};
use glam::Vec3;

// ─────────────────────────────────────────────────────────────────────────────
// Viewport (3D scene)
// ─────────────────────────────────────────────────────────────────────────────

pub fn show_viewport(ui: &mut Ui, state: &mut EditorState, path: &str) {
    let project_root = state.project_path.clone();
    state.prefab.ensure_loaded(&project_root, path);

    if let Some(err) = &state.prefab.load_error {
        ui.centered_and_justified(|ui| {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("Prefab failed to load").color(theme::RED));
                ui.label(RichText::new(err).small().color(theme::TEXT_MUTED));
            });
        });
        return;
    }

    let Some(render_shared) = state.render_shared().cloned() else {
        ui.centered_and_justified(|ui| ui.label(RichText::new("No GPU device available").color(theme::RED)));
        return;
    };

    let Some(resolved) = &state.prefab.resolved else {
        ui.centered_and_justified(|ui| ui.label(RichText::new("Loading...").color(theme::TEXT_MUTED)));
        return;
    };

    if state.prefab_scene.is_none() {
        state.prefab_scene = Some(crate::prefab_viewer::PrefabScene::new(render_shared.device.clone(), render_shared.queue.clone()));
    }

    let full_raw = prefab_resolver::resolved_to_raw_full(resolved);

    // `has_local_transform`/`next_comp_index`/`effective` describe the
    // COMMITTED state — needed to decide (at drag start) whether a local
    // `Transform3D` override needs adding, and what to seed it with.
    // `world_matrix`/`parent_world_matrix` describe where the object
    // actually IS *this frame* — computed further down from `preview_raw`
    // (which bakes in any live drag value), not from `full_raw` — otherwise
    // the gizmo's drawn position stays frozen at the pre-drag location
    // while the mesh itself visibly moves, since `full_raw` never reflects
    // an in-progress (uncommitted) drag.
    let selected_path = state.prefab.selected_path.clone();
    let local_info = selected_path.as_ref().and_then(|sel_path| {
        let raw_root = state.prefab.raw.as_ref()?;
        let local_node = prefab_types::get_node_at_path(raw_root, sel_path)?;
        let has_local_transform = local_node
            .components
            .iter()
            .any(|c| c.kind == "Transform3D");
        let next_comp_index = local_node
            .components
            .iter()
            .position(|c| c.kind == "Transform3D")
            .unwrap_or(local_node.components.len());

        let full_node = prefab_types::get_node_at_path(&full_raw, sel_path);
        let effective = full_node
            .and_then(|n| n.components.iter().find(|c| c.kind == "Transform3D"))
            .map(prefab_types::read_transform_fields)
            .unwrap_or_default();

        Some((effective, has_local_transform, next_comp_index))
    });

    let mut gizmo_mode = state.prefab.gizmo_mode;
    let mut gizmo_space = state.prefab.gizmo_space;
    let mut gizmo_drag = state.prefab.gizmo_drag.take();
    if local_info.is_none() {
        gizmo_drag = None;
    }

    // W/E/R switch gizmo mode — standard convention, matching most 3D
    // editors. Guarded on nothing currently having keyboard focus so
    // typing "w"/"e"/"r" into a rename box or a text field elsewhere in
    // the inspector doesn't accidentally change modes out from under you.
    if ui.ctx().memory(|m| m.focused().is_none()) {
        ui.ctx().input(|i| {
            if i.key_pressed(egui::Key::W) {
                gizmo_mode = GizmoMode::Translate;
            } else if i.key_pressed(egui::Key::E) {
                gizmo_mode = GizmoMode::Rotate;
            } else if i.key_pressed(egui::Key::R) {
                gizmo_mode = GizmoMode::Scale;
            }
        });
    }

    let preview_raw = match (&gizmo_drag, &selected_path) {
        (Some(drag), Some(sel_path)) if &drag.path == sel_path => {
            let mode = mode_of_drag(drag);
            let mut tree = full_raw.clone();
            if let Some(node) = prefab_types::get_node_at_path_mut(&mut tree, sel_path) {
                if let Some(comp) = node.components.iter_mut().find(|c| c.kind == "Transform3D") {
                    let mut fields = prefab_types::read_transform_fields(comp);
                    match mode {
                        GizmoMode::Translate => fields.position = drag.current_value,
                        GizmoMode::Rotate => fields.rotation = drag.current_value,
                        GizmoMode::Scale => fields.scale = drag.current_value,
                    }
                    *comp = prefab_types::write_transform_fields(comp, fields);
                } else {
                    let mut fields = prefab_types::TransformFields::default();
                    match mode {
                        GizmoMode::Translate => fields.position = drag.current_value,
                        GizmoMode::Rotate => fields.rotation = drag.current_value,
                        GizmoMode::Scale => fields.scale = drag.current_value,
                    }
                    node.components
                        .push(prefab_types::write_transform_fields(&prefab_types::default_component("Transform3D"), fields));
                }
            }
            tree
        }
        _ => full_raw.clone(),
    };

    // Now that `preview_raw` exists (reflecting any live drag), compute the
    // gizmo's actual draw position from it — this is the fix for "the
    // gizmo doesn't follow the object while dragging."
    let gizmo_target_info = selected_path
        .as_ref()
        .zip(local_info)
        .map(|(sel_path, (effective, has_local_transform, next_comp_index))| {
            let (world_matrix, parent_world_matrix) = prefab_transforms::world_matrices_for_path(&preview_raw, sel_path);
            (sel_path.clone(), world_matrix, parent_world_matrix, effective, has_local_transform, next_comp_index)
        });

    let entries = prefab_transforms::collect_render_entries(&project_root, &preview_raw);

    let scene = state.prefab_scene.as_mut().unwrap();
    scene.sync(&entries);
    if state.prefab.camera_reset_requested {
        scene.reset_camera();
        state.prefab.camera_reset_requested = false;
    }

    // Collected inside the closure below, applied to `state.prefab`
    // strictly *after* it ends — `scene` stays borrowed from
    // `state.prefab_scene` for the whole closure, so nothing in here
    // touches `state.prefab` directly to avoid relying on the borrow
    // checker's disjoint-field-capture analysis working perfectly through
    // a closure boundary (same reasoning as `asset_tree.rs`'s action queue).
    // `gizmo_mode`/`gizmo_drag` are handled the same way — taken out as
    // locals above, mutated freely inside the closure, written back after.
    let mut pending_select: Option<Option<Vec<usize>>> = None;
    let mut gizmo_actions: Vec<PrefabAction> = Vec::new();

    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("{} render entries", entries.len()))
                    .small()
                    .color(theme::TEXT_SECONDARY),
            );

            ui.separator();
            for (mode, label) in [(GizmoMode::Translate, "Move (W)"), (GizmoMode::Rotate, "Rotate (E)"), (GizmoMode::Scale, "Scale (R)")] {
                if ui.selectable_label(gizmo_mode == mode, label).clicked() {
                    gizmo_mode = mode;
                }
            }

            ui.separator();
            for (space, label) in [(crate::prefab_state::GizmoSpace::Global, "Global"), (crate::prefab_state::GizmoSpace::Local, "Local")] {
                if ui.selectable_label(gizmo_space == space, label).clicked() {
                    gizmo_space = space;
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("Reset camera").clicked() {
                    scene.reset_camera();
                }
            });
        });

        let avail = ui.available_size();
        let width = avail.x.max(1.0) as u32;
        let height = avail.y.max(1.0) as u32;
        let aspect = width as f32 / height.max(1) as f32;

        let (texture_id, size) = scene.render(&render_shared.render_state, width, height);
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
        ui.painter()
            .image(texture_id, rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);

        let view_proj = scene.view_proj(aspect);
        let camera_eye = scene.camera.eye();

        // Gizmo takes priority over camera-drag/ray-pick when it's
        // actively involved (hovering close enough to start a drag, or
        // continuing one already in progress) — it reports back via the
        // `Some(..)` return whether it owns this frame's input.
        let mut gizmo_owns_input = false;
        if let Some((gpath, world_matrix, parent_world_matrix, effective, has_local_transform, next_comp_index)) = &gizmo_target_info {
            let target = GizmoTarget {
                path: gpath,
                world_matrix: *world_matrix,
                parent_world_matrix: *parent_world_matrix,
                effective: *effective,
                has_local_transform: *has_local_transform,
                next_comp_index: *next_comp_index,
            };
            if prefab_gizmo::interact(ui, rect, view_proj, camera_eye, gizmo_mode, gizmo_space, &target, &mut gizmo_drag, &mut gizmo_actions).is_some() {
                gizmo_owns_input = true;
            }
        }

        // Marker overlay for Spine (RendererDynamic) entries — see
        // prefab_viewer.rs's doc comment on why these aren't fully rendered.
        let mut marker_click: Option<Vec<usize>> = None;
        for (world_pos, marker_path, name) in scene.markers() {
            if let Some(screen_pos) = world_to_screen(*world_pos, view_proj, rect) {
                let is_selected = selected_path.as_deref() == Some(marker_path.as_slice());
                let color = if is_selected { theme::BLUE } else { theme::ORANGE };
                ui.painter().circle_filled(screen_pos, 5.0, color);
                ui.painter()
                    .text(screen_pos + egui::vec2(8.0, -4.0), egui::Align2::LEFT_CENTER, name, egui::FontId::proportional(11.0), color);
                let marker_rect = egui::Rect::from_center_size(screen_pos, egui::vec2(14.0, 14.0));
                if !gizmo_owns_input && ui.rect_contains_pointer(marker_rect) && ui.input(|i| i.pointer.primary_clicked()) {
                    marker_click = Some(marker_path.clone());
                }
            }
        }

        if !gizmo_owns_input {
            if response.dragged() {
                scene.camera.apply_input(response.drag_delta(), 0.0);
            }
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if response.hovered() && scroll != 0.0 {
                scene.camera.apply_input(egui::Vec2::ZERO, scroll);
            }

            if let Some(path) = marker_click {
                pending_select = Some(Some(path));
            } else if response.clicked() && !response.dragged() {
                if let Some(pointer) = response.interact_pointer_pos() {
                    let local = pointer - rect.min;
                    if let Some((origin, dir)) = screen_to_ray(local, rect.size(), view_proj) {
                        pending_select = Some(scene.pick(origin, dir));
                    }
                }
            }

            let scroll_active = response.hovered() && scroll != 0.0;
            if response.dragged() || scroll_active {
                ui.ctx().request_repaint();
            }
        } else {
            // Actively dragging a gizmo handle also needs continuous
            // repaint for smooth visual feedback.
            ui.ctx().request_repaint();
        }
    });

    state.prefab.gizmo_mode = gizmo_mode;
    state.prefab.gizmo_space = gizmo_space;
    state.prefab.gizmo_drag = gizmo_drag;

    if let Some(new_selection) = pending_select {
        state
            .prefab
            .apply(PrefabAction::Select(new_selection), &project_root);
    }
    for action in gizmo_actions {
        state.prefab.apply(action, &project_root);
    }
}

fn mode_of_drag(drag: &crate::prefab_state::GizmoDrag) -> GizmoMode {
    match &drag.kind {
        crate::prefab_state::GizmoDragKind::Translate { .. } => GizmoMode::Translate,
        crate::prefab_state::GizmoDragKind::Rotate { .. } => GizmoMode::Rotate,
        crate::prefab_state::GizmoDragKind::Scale { .. } => GizmoMode::Scale,
    }
}

fn world_to_screen(world: Vec3, view_proj: glam::Mat4, rect: egui::Rect) -> Option<egui::Pos2> {
    let clip = view_proj * world.extend(1.0);
    if clip.w <= 0.0 {
        return None;
    }
    let ndc = clip.truncate() / clip.w;
    let x = rect.min.x + (ndc.x * 0.5 + 0.5) * rect.width();
    let y = rect.min.y + (1.0 - (ndc.y * 0.5 + 0.5)) * rect.height();
    Some(egui::pos2(x, y))
}

/// Unprojects a click at `local` (pixels, relative to the viewport rect) into
/// a world-space ray, given the camera's combined `view_proj`.
fn screen_to_ray(local: egui::Vec2, size: egui::Vec2, view_proj: glam::Mat4) -> Option<(Vec3, Vec3)> {
    let ndc_x = (local.x / size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (local.y / size.y) * 2.0;
    let inv = view_proj.inverse();

    let near = inv * glam::Vec4::new(ndc_x, ndc_y, 0.0, 1.0);
    let far = inv * glam::Vec4::new(ndc_x, ndc_y, 1.0, 1.0);
    if near.w == 0.0 || far.w == 0.0 {
        return None;
    }
    let near = near.truncate() / near.w;
    let far = far.truncate() / far.w;
    Some((near, (far - near).normalize()))
}

// ─────────────────────────────────────────────────────────────────────────────
// Inspector (tree + component/field editing)
// ─────────────────────────────────────────────────────────────────────────────

pub fn show_inspector(ui: &mut Ui, state: &mut EditorState, path: &str) {
    let project_root = state.project_path.clone();
    state.prefab.ensure_loaded(&project_root, path);

    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Inspector")
                .strong()
                .color(theme::TEXT_PRIMARY),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .small_button("Refresh")
                .on_hover_text("Re-resolve from disk")
                .clicked()
            {
                state.prefab.reload(&project_root);
            }
        });
    });
    ui.separator();

    if let Some(err) = &state.prefab.load_error {
        ui.colored_label(theme::RED, err);
        return;
    }
    let Some(raw) = state.prefab.raw.clone() else {
        ui.weak("Loading...");
        return;
    };

    let name = path.rsplit('/').next().unwrap_or(path);
    ui.label(RichText::new(name).strong());
    let meta = match &raw.base {
        Some(b) => format!("Prefab - base: {b}"),
        None => "Prefab".to_string(),
    };
    ui.label(RichText::new(meta).small().color(theme::TEXT_SECONDARY));
    ui.add_space(6.0);

    let mut actions = Vec::new();
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            game_object_node(ui, state, &raw, 0, &[], &mut actions, &project_root);
        });

    for action in actions {
        state.prefab.apply(action, &project_root);
    }
}

#[allow(clippy::too_many_arguments)]
fn game_object_node(ui: &mut Ui, state: &EditorState, node: &PrefabGameObjectRaw, depth: usize, path: &[usize], actions: &mut Vec<PrefabAction>, project_root: &str) {
    let path_key = format!("{path:?}");
    let is_selected = state.prefab.selected_path.as_deref() == Some(path);
    // Default EXPANDED (not just the root) — a freshly-added child needs
    // its own "+ Add Facet"/"+ Add Child" controls visible immediately, or
    // it looks like there's no way to build nested structure at all.
    // Tracked as "closed" (a prefix key) rather than "open", same
    // default-open convention `component_block` already uses.
    let closed_key = format!("closed:{path_key}");
    let is_open = !state.prefab.expanded_nodes.contains(&closed_key);

    egui::Frame::NONE
        .fill(if is_selected { theme::BG_ACTIVE } else { egui::Color32::TRANSPARENT })
        .inner_margin(egui::Margin::symmetric(0, 1))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(depth as f32 * 12.0);

                if crate::icons::chevron(ui, is_open, theme::TEXT_SECONDARY).clicked() {
                    actions.push(PrefabAction::ToggleExpand(closed_key.clone()));
                }

                let mut enabled = node.enabled;
                if ui.checkbox(&mut enabled, "").changed() {
                    actions.push(PrefabAction::SetEnabled(path.to_vec(), enabled));
                }

                let name_resp = ui.add(egui::Label::new(RichText::new(&node.name).color(if is_selected { theme::BLUE } else { theme::TEXT_PRIMARY })).sense(egui::Sense::click()));
                if name_resp.clicked() {
                    actions.push(PrefabAction::Select(if is_selected { None } else { Some(path.to_vec()) }));
                }

                if !path.is_empty() {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("x").on_hover_text("Delete").clicked() {
                            actions.push(PrefabAction::RemoveChild(path.to_vec()));
                        }
                        if ui.small_button("Dup").on_hover_text("Duplicate").clicked() {
                            actions.push(PrefabAction::DuplicateChild(path.to_vec()));
                        }
                    });
                }
            });
        });

    if !is_open {
        return;
    }

    ui.indent(("gobj_body", path.len(), path.last().copied().unwrap_or(0)), |ui| {
        // base — dropdown of .comp assets
        ui.horizontal(|ui| {
            ui.label(RichText::new("base").small().color(theme::TEXT_SECONDARY));
            if let Some(new_val) = asset_dropdown(ui, ("prefab_base", path.to_vec()), node.base.as_deref(), &[".comp"], "-- no base --", project_root) {
                actions.push(PrefabAction::SetBase(path.to_vec(), new_val));
            }
        });

        for (i, comp) in node.components.iter().enumerate() {
            component_block(ui, state, comp, i, path, actions, project_root);
        }

        add_component_button(ui, path, &node.components, actions, project_root);

        if !node.children.is_empty() {
            ui.label(
                RichText::new(format!("Children ({})", node.children.len()))
                    .small()
                    .color(theme::TEXT_MUTED),
            );
        }
        for (i, child) in node.children.iter().enumerate() {
            let mut child_path = path.to_vec();
            child_path.push(i);
            game_object_node(ui, state, child, depth + 1, &child_path, actions, project_root);
        }

        if ui.small_button("+ Add Child").clicked() {
            actions.push(PrefabAction::AddChild(path.to_vec()));
        }
    });
}

// ── Component block ─────────────────────────────────────────────────────────

fn component_block(ui: &mut Ui, state: &EditorState, comp: &PrefabComponentRaw, index: usize, path: &[usize], actions: &mut Vec<PrefabAction>, project_root: &str) {
    // Components default OPEN; we track *closed* state (prefixed key) so
    // "never toggled" reads as open without needing a separate default-set
    // pass over every component on load.
    let closed_key = format!("closed:{path:?}#{index}");
    let is_open = !state.prefab.open_components.contains(&closed_key);

    egui::Frame::NONE
        .fill(theme::BG_TERTIARY)
        .inner_margin(4)
        .corner_radius(3)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if crate::icons::chevron(ui, is_open, theme::TEXT_SECONDARY).clicked() {
                    actions.push(PrefabAction::ToggleComponentOpen(closed_key.clone()));
                }
                ui.label(RichText::new(&comp.kind).color(theme::GREEN));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("x").clicked() {
                        actions.push(PrefabAction::RemoveComponent(path.to_vec(), index));
                    }
                    if ui.small_button("v").on_hover_text("Move down").clicked() {
                        actions.push(PrefabAction::MoveComponent(path.to_vec(), index, index + 1));
                    }
                    if index > 0 && ui.small_button("^").on_hover_text("Move up").clicked() {
                        actions.push(PrefabAction::MoveComponent(path.to_vec(), index, index - 1));
                    }
                });
            });

            if !is_open {
                return;
            }

            let known_fields = crate::prefab_facets::component_fields(project_root, &comp.kind);
            let known_keys: Vec<&str> = known_fields.iter().map(|f| f.name.as_str()).collect();
            let extra_fields: Vec<FieldDescriptor> = comp
                .fields
                .iter()
                .map(|f| prefab_types::split_field(f).0)
                .filter(|k| !known_keys.contains(&k.as_str()))
                .map(|k| FieldDescriptor { name: k, kind: EntryType::Float })
                .collect();

            for field in known_fields.iter().chain(extra_fields.iter()) {
                field_row(ui, comp, field, comp.kind == "Transform2D", index, path, actions, project_root);
            }
        });
}

#[allow(clippy::too_many_arguments)]
fn field_row(ui: &mut Ui, comp: &PrefabComponentRaw, field: &FieldDescriptor, is2d: bool, comp_index: usize, path: &[usize], actions: &mut Vec<PrefabAction>, project_root: &str) {
    let value = comp.fields.iter().find_map(|f| {
        let (k, v) = prefab_types::split_field(f);
        (k == field.name).then_some(v)
    });
    let is_set = value.is_some();

    match &field.kind {
        EntryType::Vector2 | EntryType::Vector3 | EntryType::Vector4 => {
            let axes: &[&str] = match &field.kind {
                EntryType::Vector2 => &["x", "y"],
                EntryType::Vector3 => &["x", "y", "z"],
                _ => &["x", "y", "z", "w"],
            };
            // Transform fields (position/rotation/scale) get the 2D/3D
            // arity handling from prefab_types; generic Vector* facet
            // fields just use their declared arity directly.
            let is_transform_field = prefab_types::is_transform(&comp.kind) && matches!(field.name.as_str(), "position" | "rotation" | "scale");

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(&field.name)
                        .small()
                        .color(if is_set { theme::TEXT_SECONDARY } else { theme::TEXT_MUTED }),
                );
                if is_set {
                    let mut nums = prefab_types::parse_tuple(value.as_deref().unwrap_or(""));
                    nums.resize(axes.len(), 0.0);
                    let mut changed = false;
                    for (i, axis) in axes.iter().enumerate() {
                        // For a Transform2D's position/scale, only show X/Y
                        // (Z stays implicit), matching TransformRow's filter.
                        if is_transform_field && is2d && field.name != "rotation" && *axis == "z" {
                            continue;
                        }
                        ui.label(
                            RichText::new(axis.to_uppercase())
                                .small()
                                .color(theme::TEXT_MUTED),
                        );
                        let mut n = nums[i];
                        if ui.add(egui::DragValue::new(&mut n).speed(0.05)).changed() {
                            nums[i] = n;
                            changed = true;
                        }
                    }
                    if changed {
                        actions.push(PrefabAction::SetComponentField(path.to_vec(), comp_index, prefab_types::join_field(&field.name, &prefab_types::format_tuple(&nums))));
                    }
                    if ui.small_button("x").clicked() {
                        actions.push(PrefabAction::RemoveComponentField(path.to_vec(), comp_index, field.name.clone()));
                    }
                } else {
                    ui.weak("--");
                    if ui.small_button("set").clicked() {
                        actions.push(PrefabAction::SetComponentField(path.to_vec(), comp_index, prefab_types::join_field(&field.name, &prefab_types::format_tuple(&vec![0.0; axes.len()]))));
                    }
                }
            });
        }

        EntryType::Asset(suffix) => {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(&field.name)
                        .small()
                        .color(if is_set { theme::TEXT_SECONDARY } else { theme::TEXT_MUTED }),
                );
                let id_source = ("prefab_field", path.to_vec(), comp_index, field.name.clone());
                if let Some(new_val) = asset_dropdown(ui, id_source, value.as_deref().filter(|v| !v.trim().is_empty()), &[suffix.as_str()], "-- select asset --", project_root) {
                    match new_val {
                        Some(id) => actions.push(PrefabAction::SetComponentField(path.to_vec(), comp_index, prefab_types::join_field(&field.name, &id))),
                        None => actions.push(PrefabAction::RemoveComponentField(path.to_vec(), comp_index, field.name.clone())),
                    }
                }
            });
        }

        EntryType::Bool => {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(&field.name)
                        .small()
                        .color(if is_set { theme::TEXT_SECONDARY } else { theme::TEXT_MUTED }),
                );
                if is_set {
                    let mut checked = value.as_deref() == Some("true");
                    if ui.checkbox(&mut checked, "").changed() {
                        actions.push(PrefabAction::SetComponentField(path.to_vec(), comp_index, prefab_types::join_field(&field.name, if checked { "true" } else { "false" })));
                    }
                    if ui.small_button("x").clicked() {
                        actions.push(PrefabAction::RemoveComponentField(path.to_vec(), comp_index, field.name.clone()));
                    }
                } else {
                    ui.weak("--");
                    if ui.small_button("set").clicked() {
                        actions.push(PrefabAction::SetComponentField(path.to_vec(), comp_index, prefab_types::join_field(&field.name, "false")));
                    }
                }
            });
        }

        EntryType::Float | EntryType::Int => {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(&field.name)
                        .small()
                        .color(if is_set { theme::TEXT_SECONDARY } else { theme::TEXT_MUTED }),
                );
                if is_set {
                    let mut draft = value.clone().unwrap_or_default();
                    let resp = ui.text_edit_singleline(&mut draft);
                    if resp.lost_focus() && !draft.trim().is_empty() {
                        actions.push(PrefabAction::SetComponentField(path.to_vec(), comp_index, prefab_types::join_field(&field.name, &draft)));
                    }
                    if ui.small_button("x").clicked() {
                        actions.push(PrefabAction::RemoveComponentField(path.to_vec(), comp_index, field.name.clone()));
                    }
                } else {
                    ui.weak("--");
                    if ui.small_button("set").clicked() {
                        actions.push(PrefabAction::SetComponentField(path.to_vec(), comp_index, prefab_types::join_field(&field.name, "0")));
                    }
                }
            });
        }
    }
}

fn add_component_button(ui: &mut Ui, path: &[usize], existing: &[PrefabComponentRaw], actions: &mut Vec<PrefabAction>, project_root: &str) {
    let existing_kinds: Vec<&str> = existing.iter().map(|c| c.kind.as_str()).collect();
    let all_types = crate::prefab_facets::all_component_types(project_root);

    ui.menu_button("+ Add Facet", |ui| {
        let mut any = false;
        for kind in &all_types {
            if existing_kinds.contains(&kind.as_str()) {
                continue;
            }
            any = true;
            if ui.button(kind).clicked() {
                actions.push(PrefabAction::AddComponent(path.to_vec(), kind.clone()));
                ui.close_menu();
            }
        }
        if !any {
            ui.weak("All facets present");
        }
    });
}

// ── Asset dropdown (base / renderer-asset fields) ───────────────────────────
// Port of AssetDropdown.tsx. Returns `Some(new_value)` if the selection
// changed this frame (`Some(None)` = cleared), `None` if unchanged.
fn asset_dropdown(ui: &mut Ui, id_source: impl std::hash::Hash, value: Option<&str>, accepts: &[&str], placeholder: &str, project_root: &str) -> Option<Option<String>> {
    let entries = fs_ops::read_manifest(project_root);
    let current_id: Option<i16> = value.and_then(|v| v.trim().parse().ok());
    let current_entry = entries.iter().find(|e| Some(e.id) == current_id);
    let display = match (current_entry, current_id) {
        (Some(e), _) => e.name.clone(),
        (None, Some(id)) => format!("unknown ({id})"),
        (None, None) => placeholder.to_string(),
    };

    let mut result = None;
    egui::ComboBox::from_id_salt(id_source)
        .selected_text(display)
        .show_ui(ui, |ui| {
            if current_entry.is_some()
                && ui
                    .selectable_label(false, RichText::new("-- clear --").italics())
                    .clicked()
            {
                result = Some(None);
            }
            for entry in &entries {
                let ext = fs_ops::file_ext(&entry.uri);
                if !accepts.contains(&ext.as_str()) {
                    continue;
                }
                let label = format!("{}  ({})", entry.name, entry.uri.rsplit('/').next().unwrap_or(&entry.uri));
                if ui
                    .selectable_label(Some(entry.id) == current_id, label)
                    .clicked()
                {
                    result = Some(Some(entry.id.to_string()));
                }
            }
        });
    result
}
