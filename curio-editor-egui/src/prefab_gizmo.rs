//! Move/rotate/scale gizmo for the prefab 3D viewport.
//!
//! **Design choice, worth knowing:** this draws the gizmo as a 2D overlay
//! (lines/arrowheads/circles projected from world space via the camera's
//! `view_proj`, painted with `ui.painter()`) rather than building actual 3D
//! gizmo geometry into `prefab_viewer.rs`'s wgpu pipeline. This is
//! significantly simpler — it reuses the same world-to-screen projection
//! `prefab_tab.rs` already has for the Spine placeholder markers — at the
//! cost of the handles not being depth-tested against the scene geometry
//! (a handle will always draw on top, even "behind" a mesh from the current
//! camera angle). For a prefab-composition preview tool this is a
//! reasonable tradeoff; a AAA-grade gizmo would want real 3D geometry.
//!
//! **Axis conventions:**
//! - Translate handles are **world-aligned** (X/Y/Z always point the same
//!   way on screen regardless of the object's rotation) — this is the
//!   common "global" gizmo mode default in most editors.
//! - Rotate and Scale handles are **object-local** (transformed by the
//!   object's own world rotation) — rotating "around local X" and scaling
//!   "along local X" are the meaningful operations for those, unlike
//!   translation.
//!
//! **Known simplification:** rotation drags add degrees directly to the
//! Euler `rotation` field's corresponding axis component. This works well
//! from a fresh/zero rotation, which is the common case, but naive
//! Euler-component addition doesn't correctly compose once multiple axes
//! already have non-zero rotation (a real gizmo would compose quaternion
//! deltas). Flagged here rather than silently wrong.

use crate::prefab_state::{GizmoDrag, GizmoDragKind, GizmoMode, PrefabAction};
use crate::prefab_types::{self, TransformFields};
use eframe::egui::{self, Color32, Pos2, Stroke, Ui};
use glam::{Mat4, Vec2 as GVec2, Vec3};

const HANDLE_HIT_PX: f32 = 10.0;
const SCALE_SENSITIVITY: f32 = 0.01;
const MIN_SCALE: f32 = 0.01;

const AXIS_COLORS: [Color32; 3] = [Color32::from_rgb(0xe0, 0x5a, 0x5a), Color32::from_rgb(0x5a, 0xd0, 0x6a), Color32::from_rgb(0x5a, 0x8c, 0xe0)];

fn field_name(mode: GizmoMode) -> &'static str {
    match mode {
        GizmoMode::Translate => "position",
        GizmoMode::Rotate => "rotation",
        GizmoMode::Scale => "scale",
    }
}

fn to_pos2(p: GVec2) -> Pos2 {
    egui::pos2(p.x, p.y)
}
fn to_gvec2(p: Pos2) -> GVec2 {
    GVec2::new(p.x, p.y)
}

fn world_to_screen(world: Vec3, view_proj: Mat4, rect: egui::Rect) -> Option<GVec2> {
    let clip = view_proj * world.extend(1.0);
    if clip.w <= 0.0 {
        return None;
    }
    let ndc = clip.truncate() / clip.w;
    let x = rect.min.x + (ndc.x * 0.5 + 0.5) * rect.width();
    let y = rect.min.y + (1.0 - (ndc.y * 0.5 + 0.5)) * rect.height();
    Some(GVec2::new(x, y))
}

fn dist_point_to_segment(p: GVec2, a: GVec2, b: GVec2) -> f32 {
    let ab = b - a;
    let len2 = ab.length_squared();
    if len2 < 1e-6 {
        return (p - a).length();
    }
    let t = ((p - a).dot(ab) / len2).clamp(0.0, 1.0);
    (p - (a + ab * t)).length()
}

fn get_axis(v: &prefab_types::Vec3, axis: usize) -> f32 {
    match axis {
        0 => v.x,
        1 => v.y,
        _ => v.z,
    }
}
fn set_axis(v: &mut prefab_types::Vec3, axis: usize, value: f32) {
    match axis {
        0 => v.x = value,
        1 => v.y = value,
        _ => v.z = value,
    }
}

fn fields_with(base: TransformFields, mode: GizmoMode, value: prefab_types::Vec3) -> TransformFields {
    match mode {
        GizmoMode::Translate => TransformFields { position: value, ..base },
        GizmoMode::Rotate => TransformFields { rotation: value, ..base },
        GizmoMode::Scale => TransformFields { scale: value, ..base },
    }
}

/// Everything the gizmo needs to know about the currently-selected object,
/// gathered by `prefab_tab.rs` before calling `interact`.
pub struct GizmoTarget<'a> {
    pub path: &'a [usize],
    pub world_matrix: Mat4,
    pub parent_world_matrix: Mat4,
    /// Effective (possibly-inherited) transform — this is the baseline a
    /// fresh drag starts from, and what seeds a new local `Transform3D`
    /// override if the object doesn't have its own yet, so the object
    /// doesn't visually jump the instant you start dragging.
    pub effective: TransformFields,
    /// Whether the node has its own local `Transform3D` component already.
    pub has_local_transform: bool,
    /// Index that component has now, or will have immediately after
    /// `AddComponentWithFields` if `has_local_transform` is false (i.e.
    /// `node.components.len()` at the time of the check).
    pub next_comp_index: usize,
}

/// Draws the gizmo for `mode` and handles hit-testing/dragging.
///
/// Returns `Some(fields)` if a gizmo drag is active this frame (started or
/// continuing) — `fields` is the live value to use for this frame's 3D
/// preview (only the field `mode` edits is meaningful; the others are
/// copied from `target.effective` as a base). The caller should skip
/// camera-drag and ray-pick handling for this frame's input when this
/// returns `Some(..)` (the gizmo owns the input this frame). Returns `None`
/// if no gizmo drag is in progress.
pub fn interact(ui: &mut Ui, rect: egui::Rect, view_proj: Mat4, camera_eye: Vec3, mode: GizmoMode, target: &GizmoTarget, gizmo_drag: &mut Option<GizmoDrag>, actions: &mut Vec<PrefabAction>) -> Option<TransformFields> {
    // A stale drag (selection changed mid-drag, shouldn't normally happen
    // since selection changes go through the same deferred-action queue,
    // but defensive) — drop it rather than apply it to the wrong object.
    if gizmo_drag.as_ref().is_some_and(|d| d.path != target.path) {
        *gizmo_drag = None;
    }

    let origin = target.world_matrix.transform_point3(Vec3::ZERO);
    let handle_length = ((camera_eye - origin).length() * 0.2).max(0.05);
    let Some(origin_screen) = world_to_screen(origin, view_proj, rect) else {
        return gizmo_drag
            .as_ref()
            .map(|d| fields_with(target.effective, mode, d.current_value));
    };

    let axis_dirs: [Vec3; 3] = match mode {
        GizmoMode::Translate => [Vec3::X, Vec3::Y, Vec3::Z],
        GizmoMode::Rotate | GizmoMode::Scale => [
            target
                .world_matrix
                .transform_vector3(Vec3::X)
                .normalize_or_zero(),
            target
                .world_matrix
                .transform_vector3(Vec3::Y)
                .normalize_or_zero(),
            target
                .world_matrix
                .transform_vector3(Vec3::Z)
                .normalize_or_zero(),
        ],
    };

    let pointer = ui.input(|i| i.pointer.latest_pos());
    let pressed_now = ui.input(|i| i.pointer.primary_pressed());
    let released_now = ui.input(|i| i.pointer.primary_released());

    // ── Draw + hit-test each axis ────────────────────────────────────────────
    let mut closest_hit: Option<usize> = None;
    let mut closest_dist = f32::MAX;

    for (axis, &dir) in axis_dirs.iter().enumerate() {
        let color = AXIS_COLORS[axis];
        let is_dragging_this = gizmo_drag
            .as_ref()
            .is_some_and(|d| d.axis == axis && d.path == target.path);

        match mode {
            GizmoMode::Translate | GizmoMode::Scale => {
                let tip_world = origin + dir * handle_length;
                let Some(tip_screen) = world_to_screen(tip_world, view_proj, rect) else { continue };

                let dist_to_pointer = pointer.map(|p| dist_point_to_segment(to_gvec2(p), origin_screen, tip_screen));
                if let Some(d) = dist_to_pointer {
                    if d < HANDLE_HIT_PX && d < closest_dist {
                        closest_dist = d;
                        closest_hit = Some(axis);
                    }
                }
                let hovered = dist_to_pointer.is_some_and(|d| d < HANDLE_HIT_PX);

                let stroke_w = if is_dragging_this || hovered { 3.5 } else { 2.0 };
                ui.painter()
                    .line_segment([to_pos2(origin_screen), to_pos2(tip_screen)], Stroke::new(stroke_w, color));

                if mode == GizmoMode::Translate {
                    let dir2 = (tip_screen - origin_screen).normalize_or_zero();
                    let perp = GVec2::new(-dir2.y, dir2.x) * 5.0;
                    let tip = to_pos2(tip_screen + dir2 * 6.0);
                    let base_a = to_pos2(tip_screen - dir2 * 4.0 + perp);
                    let base_b = to_pos2(tip_screen - dir2 * 4.0 - perp);
                    ui.painter()
                        .add(egui::Shape::convex_polygon(vec![tip, base_a, base_b], color, Stroke::NONE));
                } else {
                    ui.painter()
                        .rect_filled(egui::Rect::from_center_size(to_pos2(tip_screen), egui::vec2(8.0, 8.0)), 1.0, color);
                }
            }

            GizmoMode::Rotate => {
                const SEGMENTS: usize = 32;
                let helper = if dir.dot(Vec3::Y).abs() > 0.99 { Vec3::X } else { Vec3::Y };
                let u = dir.cross(helper).normalize_or_zero();
                let v = dir.cross(u).normalize_or_zero();

                let mut screen_points = Vec::with_capacity(SEGMENTS + 1);
                for i in 0..=SEGMENTS {
                    let t = (i as f32 / SEGMENTS as f32) * std::f32::consts::TAU;
                    let p = origin + (u * t.cos() + v * t.sin()) * handle_length;
                    if let Some(sp) = world_to_screen(p, view_proj, rect) {
                        screen_points.push(to_pos2(sp));
                    }
                }

                let mut min_dist = f32::MAX;
                if let Some(p) = pointer {
                    let pg = to_gvec2(p);
                    for w in screen_points.windows(2) {
                        min_dist = min_dist.min(dist_point_to_segment(pg, to_gvec2(w[0]), to_gvec2(w[1])));
                    }
                    if min_dist < HANDLE_HIT_PX && min_dist < closest_dist {
                        closest_dist = min_dist;
                        closest_hit = Some(axis);
                    }
                }
                let hovered = min_dist < HANDLE_HIT_PX;

                let stroke_w = if is_dragging_this || hovered { 3.0 } else { 1.5 };
                ui.painter()
                    .add(egui::Shape::line(screen_points, Stroke::new(stroke_w, color)));
            }
        }
    }

    // ── Start a new drag ─────────────────────────────────────────────────────
    if gizmo_drag.is_none() {
        if let (true, Some(axis), Some(p)) = (pressed_now, closest_hit, pointer) {
            if !target.has_local_transform {
                let seed = prefab_types::write_transform_fields(&prefab_types::default_component("Transform3D"), target.effective);
                actions.push(PrefabAction::AddComponentWithFields(target.path.to_vec(), "Transform3D".to_string(), seed.fields));
            }

            let start_value = match mode {
                GizmoMode::Translate => target.effective.position,
                GizmoMode::Rotate => target.effective.rotation,
                GizmoMode::Scale => target.effective.scale,
            };

            let dir = axis_dirs[axis];
            let tip_screen = world_to_screen(origin + dir * handle_length, view_proj, rect).unwrap_or(origin_screen);
            let screen_axis_dir = (tip_screen - origin_screen).normalize_or_zero();

            let kind = match mode {
                GizmoMode::Translate => {
                    let screen_units_per_world = ((tip_screen - origin_screen).length() / handle_length).max(0.001);
                    GizmoDragKind::Translate {
                        world_axis_dir: dir,
                        screen_axis_dir,
                        screen_units_per_world,
                    }
                }
                GizmoMode::Scale => GizmoDragKind::Scale { screen_axis_dir },
                GizmoMode::Rotate => {
                    let pg = to_gvec2(p);
                    let angle = (pg.y - origin_screen.y).atan2(pg.x - origin_screen.x);
                    GizmoDragKind::Rotate { start_mouse_angle: angle }
                }
            };

            *gizmo_drag = Some(GizmoDrag {
                path: target.path.to_vec(),
                comp_index: target.next_comp_index,
                axis,
                start_value,
                current_value: start_value,
                start_mouse: to_gvec2(p),
                kind,
            });
        }
    }

    // ── Continue / end the active drag ───────────────────────────────────────
    if let Some(drag) = gizmo_drag.as_mut() {
        if let Some(p) = pointer {
            let mouse = to_gvec2(p);
            let mouse_delta = mouse - drag.start_mouse;

            drag.current_value = match &drag.kind {
                GizmoDragKind::Translate {
                    world_axis_dir,
                    screen_axis_dir,
                    screen_units_per_world,
                } => {
                    let pixels_along_axis = mouse_delta.dot(*screen_axis_dir);
                    let world_units = pixels_along_axis / *screen_units_per_world;
                    let world_delta = *world_axis_dir * world_units;
                    let local_delta = target
                        .parent_world_matrix
                        .inverse()
                        .transform_vector3(world_delta);
                    prefab_types::Vec3 {
                        x: drag.start_value.x + local_delta.x,
                        y: drag.start_value.y + local_delta.y,
                        z: drag.start_value.z + local_delta.z,
                    }
                }
                GizmoDragKind::Scale { screen_axis_dir } => {
                    let pixels_along_axis = mouse_delta.dot(*screen_axis_dir);
                    let delta = pixels_along_axis * SCALE_SENSITIVITY;
                    let mut v = drag.start_value;
                    let v2 = v.clone();
                    set_axis(&mut v, drag.axis, (get_axis(&v2, drag.axis) + delta).max(MIN_SCALE));
                    v
                }
                GizmoDragKind::Rotate { start_mouse_angle } => {
                    let angle = (mouse.y - origin_screen.y).atan2(mouse.x - origin_screen.x);
                    let delta_deg = (angle - start_mouse_angle).to_degrees();
                    let mut v = drag.start_value;
                    let v2 = v.clone();
                    set_axis(&mut v, drag.axis, get_axis(&v2, drag.axis) + delta_deg);
                    v
                }
            };
        }

        // Pull everything needed out into locals *before* possibly
        // clearing `*gizmo_drag` below — keeps the "is `drag`'s borrow
        // still live when we write through `gizmo_drag`" question moot
        // rather than relying on NLL to figure it out.
        let live = drag.current_value;
        let comp_index = drag.comp_index;
        let drag_path = drag.path.clone();

        if released_now {
            let vec_str = prefab_types::format_vec3(live);
            actions.push(PrefabAction::SetComponentField(drag_path, comp_index, prefab_types::join_field(field_name(mode), &vec_str)));
            *gizmo_drag = None;
        }

        return Some(fields_with(target.effective, mode, live));
    }

    None
}
