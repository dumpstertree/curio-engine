//! State backing the prefab editor (`.comp` files) — combines what
//! `PrefabLoader.tsx` (load/resolve/save orchestration) and
//! `PrefabInspectorView.tsx` (tree/component/field edit actions) did.
//!
//! Same action-queue shape as `asset_state.rs`: the tree/inspector widget
//! walks `raw` immutably each frame and returns a list of `PrefabAction`s,
//! applied afterward by `PrefabState::apply`. Every apply re-resolves the
//! `base:` chain and writes the file back to disk — there's no debounce
//! timer (the original TS had a 300ms one); instead, like the asset tree's
//! rename field, edits commit on blur/Enter/dropdown-select rather than
//! per-keystroke, so there's no need to throttle saves at all.

use crate::fs_ops;
use crate::prefab_resolver::{self, ResolvedGameObject};
use crate::prefab_types::{self, PrefabComponentRaw, PrefabGameObjectRaw};
use std::collections::HashSet;

pub struct PrefabState {
    pub file_path: Option<String>,
    pub raw: Option<PrefabGameObjectRaw>,
    pub resolved: Option<ResolvedGameObject>,
    pub load_error: Option<String>,

    pub selected_path: Option<Vec<usize>>,
    pub expanded_nodes: HashSet<String>,
    pub open_components: HashSet<String>,

    pub camera_reset_requested: bool,

    /// Which handles the 3D viewport's gizmo currently shows for the
    /// selected object — move/rotate/scale, chosen via the small toolbar in
    /// `prefab_tab.rs::show_viewport`.
    pub gizmo_mode: GizmoMode,
    /// Set while the user is actively dragging a gizmo handle; `None`
    /// otherwise. See `GizmoDrag`'s doc comment and `prefab_gizmo.rs`.
    pub gizmo_drag: Option<GizmoDrag>,
}

impl PrefabState {
    pub fn new() -> Self {
        Self {
            file_path: None,
            raw: None,
            resolved: None,
            load_error: None,
            selected_path: None,
            expanded_nodes: HashSet::new(),
            open_components: HashSet::new(),
            camera_reset_requested: false,
            gizmo_mode: GizmoMode::default(),
            gizmo_drag: None,
        }
    }

    /// Call each frame a `.comp` file is selected in the Asset tab. No-ops
    /// if the same file is already loaded.
    pub fn ensure_loaded(&mut self, project_root: &str, path: &str) {
        if self.file_path.as_deref() == Some(path) {
            return;
        }
        self.file_path = Some(path.to_string());
        self.selected_path = None;
        self.load_error = None;

        match fs_ops::read_file_bytes(path) {
            Ok(bytes) => match prefab_resolver::load_raw(&bytes) {
                Ok(raw) => {
                    self.resolved = Some(prefab_resolver::resolve_node(project_root, &raw, &mut HashSet::new()));
                    self.raw = Some(raw);
                }
                Err(e) => self.load_error = Some(e),
            },
            Err(e) => self.load_error = Some(format!("Failed to read file: {e}")),
        }
    }

    pub fn reload(&mut self, project_root: &str) {
        let Some(path) = self.file_path.clone() else { return };
        self.file_path = None;
        self.ensure_loaded(project_root, &path);
    }

    fn save_and_resolve(&mut self, project_root: &str) {
        let Some(raw) = &self.raw else { return };
        self.resolved = Some(prefab_resolver::resolve_node(project_root, raw, &mut HashSet::new()));

        if let (Some(path), Ok(text)) = (&self.file_path, prefab_resolver::dump_raw(raw)) {
            if let Err(e) = fs_ops::write_file_text(path, &text) {
                eprintln!("[PrefabState] save failed: {e}");
            }
        }
    }

    pub fn apply(&mut self, action: PrefabAction, project_root: &str) {
        // Non-mutating actions (UI-only state) short-circuit before touching `raw`.
        match action {
            PrefabAction::ToggleExpand(key) => {
                if !self.expanded_nodes.remove(&key) {
                    self.expanded_nodes.insert(key);
                }
                return;
            }
            PrefabAction::ToggleComponentOpen(key) => {
                if !self.open_components.remove(&key) {
                    self.open_components.insert(key);
                }
                return;
            }
            PrefabAction::Select(path) => {
                self.selected_path = path;
                return;
            }
            PrefabAction::RequestCameraReset => {
                self.camera_reset_requested = true;
                return;
            }
            _ => {}
        }

        let Some(raw) = &self.raw else { return };
        let mut new_raw = raw.clone();

        match action {
            PrefabAction::SetEnabled(path, enabled) => {
                if let Some(node) = prefab_types::get_node_at_path_mut(&mut new_raw, &path) {
                    node.enabled = enabled;
                }
            }
            PrefabAction::SetName(path, name) => {
                if let Some(node) = prefab_types::get_node_at_path_mut(&mut new_raw, &path) {
                    node.name = name;
                }
            }
            PrefabAction::SetBase(path, base) => {
                if let Some(node) = prefab_types::get_node_at_path_mut(&mut new_raw, &path) {
                    node.base = base;
                }
            }
            PrefabAction::AddChild(path) => {
                if let Some(node) = prefab_types::get_node_at_path_mut(&mut new_raw, &path) {
                    node.children.push(prefab_types::default_game_object("New GameObject"));
                }
            }
            PrefabAction::RemoveChild(path) => {
                if let Some((parent_path, idx)) = split_last(&path) {
                    if let Some(parent) = prefab_types::get_node_at_path_mut(&mut new_raw, &parent_path) {
                        if idx < parent.children.len() {
                            parent.children.remove(idx);
                        }
                    }
                }
                if self.selected_path.as_deref() == Some(path.as_slice()) {
                    self.selected_path = None;
                }
            }
            PrefabAction::DuplicateChild(path) => {
                if let Some((parent_path, idx)) = split_last(&path) {
                    if let Some(parent) = prefab_types::get_node_at_path_mut(&mut new_raw, &parent_path) {
                        if let Some(node) = parent.children.get(idx).cloned() {
                            let mut dup = node;
                            dup.name = format!("{}_1", dup.name);
                            parent.children.insert(idx + 1, dup);
                        }
                    }
                }
            }
            PrefabAction::AddComponent(path, kind) => {
                if let Some(node) = prefab_types::get_node_at_path_mut(&mut new_raw, &path) {
                    node.components.push(prefab_types::default_component(&kind));
                }
            }
            PrefabAction::AddComponentWithFields(path, kind, fields) => {
                if let Some(node) = prefab_types::get_node_at_path_mut(&mut new_raw, &path) {
                    node.components.push(PrefabComponentRaw { kind, fields });
                }
            }
            PrefabAction::RemoveComponent(path, comp_index) => {
                if let Some(node) = prefab_types::get_node_at_path_mut(&mut new_raw, &path) {
                    if comp_index < node.components.len() {
                        node.components.remove(comp_index);
                    }
                }
            }
            PrefabAction::MoveComponent(path, from, to) => {
                if let Some(node) = prefab_types::get_node_at_path_mut(&mut new_raw, &path) {
                    if from < node.components.len() && to < node.components.len() {
                        let comp = node.components.remove(from);
                        node.components.insert(to, comp);
                    }
                }
            }
            PrefabAction::SetComponentField(path, comp_index, raw_field) => {
                if let Some(node) = prefab_types::get_node_at_path_mut(&mut new_raw, &path) {
                    if let Some(comp) = node.components.get_mut(comp_index) {
                        let key = prefab_types::split_field(&raw_field).0;
                        comp.fields.retain(|f| prefab_types::split_field(f).0 != key);
                        comp.fields.push(raw_field);
                    }
                }
            }
            PrefabAction::RemoveComponentField(path, comp_index, key) => {
                if let Some(node) = prefab_types::get_node_at_path_mut(&mut new_raw, &path) {
                    if let Some(comp) = node.components.get_mut(comp_index) {
                        comp.fields.retain(|f| prefab_types::split_field(f).0 != key);
                    }
                }
            }
            PrefabAction::ToggleExpand(_) | PrefabAction::ToggleComponentOpen(_) | PrefabAction::Select(_) | PrefabAction::RequestCameraReset => unreachable!("handled above"),
        }

        self.raw = Some(new_raw);
        self.save_and_resolve(project_root);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Gizmo state — see `prefab_gizmo.rs` for the drawing/interaction logic.
// Kept here (not in `prefab_gizmo.rs`) so `PrefabState` stays the single
// owner of all prefab-editing state, matching `selected_path`/
// `expanded_nodes`/etc. Deliberately free of `egui` types (uses `glam::Vec2`
// for screen coordinates) to keep this file's dependency footprint the same
// as the rest of the module — `prefab_gizmo.rs` converts at its boundary.
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoMode {
    Translate,
    Rotate,
    Scale,
}

impl Default for GizmoMode {
    fn default() -> Self {
        GizmoMode::Translate
    }
}

/// Mode-specific data captured once at drag start, needed to convert mouse
/// movement into the right kind of delta each frame.
#[derive(Debug, Clone)]
pub enum GizmoDragKind {
    /// `world_axis_dir` is one of the world X/Y/Z unit vectors (translate
    /// handles are world-aligned, not object-local — see `prefab_gizmo.rs`).
    /// `screen_units_per_world` is precomputed at drag start: how many
    /// screen pixels correspond to one world unit of movement along this
    /// axis, from the object's current position and the camera's
    /// projection — lets translate convert screen-space mouse delta into
    /// an accurate world-space distance.
    Translate { world_axis_dir: glam::Vec3, screen_axis_dir: glam::Vec2, screen_units_per_world: f32 },
    /// Scale handles are the object's own local axes (so scaling "along X"
    /// always means the object's local X, regardless of its rotation).
    /// Scale deltas are NOT derived from world distance (there's no
    /// coherent "world unit of scale" once parent rotation/non-uniform
    /// scale are involved) — just a direct pixel-delta-times-sensitivity,
    /// the same simplification most simple gizmo implementations use.
    Scale { screen_axis_dir: glam::Vec2 },
    /// Rotate handles are also object-local axes, but the drag itself is
    /// angle-based: the angle (radians) from the object's projected screen
    /// center to the mouse, at drag start.
    Rotate { start_mouse_angle: f32 },
}

/// An in-progress gizmo drag. Lives on `PrefabState` so it persists across
/// frames; the *actual* file write only happens once, when the drag ends
/// (mouse released) — see `prefab_gizmo.rs`. `current_value` is recomputed
/// every frame the drag is active and is what drives the live 3D preview
/// (`prefab_tab.rs` bakes it into a transient copy of the tree before
/// calling `PrefabScene::sync`, without touching `raw`/disk).
#[derive(Debug, Clone)]
pub struct GizmoDrag {
    pub path: Vec<usize>,
    /// Index the edited `Transform3D` component has (or will have, if it
    /// was just added this same drag) in the node's own `components`.
    pub comp_index: usize,
    /// 0/1/2 = X/Y/Z.
    pub axis: usize,
    pub start_value: prefab_types::Vec3,
    pub current_value: prefab_types::Vec3,
    pub start_mouse: glam::Vec2,
    pub kind: GizmoDragKind,
}

fn split_last(path: &[usize]) -> Option<(Vec<usize>, usize)> {
    if path.is_empty() {
        return None;
    }
    Some((path[..path.len() - 1].to_vec(), path[path.len() - 1]))
}

/// `Fn(&PrefabGameObjectRaw) -> PrefabGameObjectRaw` type actions are collected
/// while walking the tree this frame, applied afterward — same reasoning as
/// `asset_state.rs::TreeAction`.
pub enum PrefabAction {
    ToggleExpand(String),
    ToggleComponentOpen(String),
    Select(Option<Vec<usize>>),
    RequestCameraReset,

    SetEnabled(Vec<usize>, bool),
    SetName(Vec<usize>, String),
    SetBase(Vec<usize>, Option<String>),
    AddChild(Vec<usize>),
    RemoveChild(Vec<usize>),
    DuplicateChild(Vec<usize>),

    AddComponent(Vec<usize>, String),
    /// Same as `AddComponent`, but seeds initial fields instead of leaving
    /// it empty — used by the gizmo when it needs to add a `Transform3D` to
    /// a node that only had one via inheritance, so the new local override
    /// starts from the object's current *effective* transform instead of
    /// resetting it to the origin (see `prefab_gizmo.rs`).
    AddComponentWithFields(Vec<usize>, String, Vec<String>),
    RemoveComponent(Vec<usize>, usize),
    MoveComponent(Vec<usize>, usize, usize),
    SetComponentField(Vec<usize>, usize, String),
    RemoveComponentField(Vec<usize>, usize, String),
}
