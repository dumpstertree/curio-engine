//! State backing the Asset tab's file tree — port of `AssetFileTree.tsx`.
//!
//! React's version re-fetches a directory's children lazily per `TreeNode`
//! (`useState` + `useEffect(loadChildren, [refresh])`) and keeps
//! selection/drag/rename/delete state in the parent. Same shape here: each
//! `TreeNode` caches its children once loaded and `AssetState` holds the
//! cross-cutting interaction state. Because egui is immediate-mode, the tree
//! widget (`panels/asset_tree.rs`) walks this structure fresh every frame
//! and returns a list of `TreeAction`s, which `AssetState::apply` then
//! executes — this avoids fighting the borrow checker over mutating the
//! tree while also reading it mid-traversal.

use crate::fs_ops::{self, DirEntry, MetaFile};
use std::sync::mpsc::{Receiver, TryRecvError};

// ─────────────────────────────────────────────────────────────────────────────
// Tree node
// ─────────────────────────────────────────────────────────────────────────────

pub struct TreeNode {
    pub entry: DirEntry,
    pub expanded: bool,
    pub loaded: bool,
    pub children: Vec<TreeNode>,
    /// `None` until `get_or_create_meta` has run for this path (files only).
    pub meta: Option<MetaFile>,
}

impl TreeNode {
    fn new(entry: DirEntry) -> Self {
        Self {
            entry,
            expanded: false,
            loaded: false,
            children: Vec::new(),
            meta: None,
        }
    }

    fn load_children(&mut self) {
        self.children = fs_ops::list_dir(&self.entry.path)
            .unwrap_or_default()
            .into_iter()
            .map(TreeNode::new)
            .collect();
        self.loaded = true;
    }

    fn ensure_meta(&mut self) {
        if self.meta.is_none() && !self.entry.is_dir {
            self.meta = Some(fs_ops::get_or_create_meta(&self.entry.path));
        }
    }

    fn find_mut(&mut self, path: &str) -> Option<&mut TreeNode> {
        if self.entry.path == path {
            return Some(self);
        }
        for child in &mut self.children {
            if let Some(found) = child.find_mut(path) {
                return Some(found);
            }
        }
        None
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Interaction state
// ─────────────────────────────────────────────────────────────────────────────

pub struct AssetState {
    pub roots: Vec<TreeNode>,
    pub roots_loaded: bool,
    pub load_error: Option<String>,

    pub selected_path: Option<String>,

    // Note: there's no `drag_path` field anymore — which item is being
    // dragged is now tracked by egui's own `DragAndDrop` plugin (queried
    // live via `egui::DragAndDrop::payload` each frame in `asset_tree.rs`)
    // instead of hand-rolled state here. `drop_target` still lives here
    // since it's used to decide a row's highlight *before* that row is
    // drawn each frame, which needs last-frame's value the same way
    // `selected_path` does.
    pub drop_target: Option<String>,

    pub renaming_path: Option<String>,
    pub rename_draft: String,

    pub confirming_delete_path: Option<String>,

    // The native file picker (`fs_ops::pick_file`) blocks the calling
    // thread until the user closes the dialog. Calling it directly on the
    // main/UI thread stalls the whole event loop for as long as the dialog
    // is open — long enough that the window manager decides the app has
    // hung and offers to force-quit it. Running it on a background thread
    // and polling this channel each frame (see `poll_import`) keeps the UI
    // thread — and so the window's responsiveness — unaffected.
    pending_import: Option<(String, Receiver<Option<String>>)>,
}

impl AssetState {
    pub fn new() -> Self {
        Self {
            roots: Vec::new(),
            roots_loaded: false,
            load_error: None,
            selected_path: None,
            drop_target: None,
            renaming_path: None,
            rename_draft: String::new(),
            confirming_delete_path: None,
            pending_import: None,
        }
    }

    /// Call once per frame before rendering the tree — (re)loads the root
    /// listing if it hasn't been loaded yet.
    pub fn ensure_roots(&mut self, project_root: &str) {
        if self.roots_loaded {
            return;
        }
        let root = fs_ops::assets_root(project_root);
        match fs_ops::list_dir(&root) {
            Ok(entries) => {
                self.roots = entries.into_iter().map(TreeNode::new).collect();
                self.load_error = None;
            }
            Err(e) => self.load_error = Some(e),
        }
        self.roots_loaded = true;
    }

    fn refresh_dir(&mut self, dir: &str) {
        for root in &mut self.roots {
            if let Some(node) = root.find_mut(dir) {
                node.load_children();
                return;
            }
        }
        // `dir` is the assets root itself — reload the top-level listing.
        self.roots_loaded = false;
    }

    /// Call once per frame — completes an in-flight `Import` once the
    /// background thread the file picker is running on (see `apply`'s
    /// `Import` handler) reports back that the user picked a file or
    /// cancelled the dialog.
    pub fn poll_import(&mut self, project_root: &str) {
        // Borrows `self.pending_import` only within this expression — the
        // result is fully owned, so there's no lingering borrow stopping
        // `self` from being mutated below.
        let polled = self
            .pending_import
            .as_ref()
            .map(|(dir, rx)| (dir.clone(), rx.try_recv()));
        let Some((target_dir, poll_result)) = polled else { return };

        match poll_result {
            Ok(picked) => {
                self.pending_import = None;
                let Some(src) = picked else { return }; // user cancelled the dialog
                let name = src.rsplit('/').next().unwrap_or(&src).to_string();
                let dst = fs_ops::resolve_conflict(&target_dir, &name);
                if fs_ops::copy_file(&src, &dst).is_ok() {
                    fs_ops::get_or_create_meta(&dst);
                    self.refresh_dir(&target_dir);
                    let _ = fs_ops::rebuild_manifest(project_root);
                }
            }
            Err(TryRecvError::Empty) => {} // still picking
            Err(TryRecvError::Disconnected) => self.pending_import = None,
        }
    }

    pub fn apply(&mut self, action: TreeAction, project_root: &str) {
        match action {
            TreeAction::ToggleExpand(path) => {
                for root in &mut self.roots {
                    if let Some(node) = root.find_mut(&path) {
                        if !node.expanded && !node.loaded {
                            node.load_children();
                        }
                        node.expanded = !node.expanded;
                        break;
                    }
                }
            }

            TreeAction::Select(path) => self.selected_path = Some(path),

            TreeAction::EnsureMeta(path) => {
                for root in &mut self.roots {
                    if let Some(node) = root.find_mut(&path) {
                        node.ensure_meta();
                        break;
                    }
                }
            }

            TreeAction::ToggleIncluded(path) => {
                let mut new_meta = None;
                for root in &mut self.roots {
                    if let Some(node) = root.find_mut(&path) {
                        node.ensure_meta();
                        if let Some(meta) = &mut node.meta {
                            meta.included = !meta.included;
                            new_meta = Some(meta.clone());
                        }
                        break;
                    }
                }
                if let Some(meta) = new_meta {
                    let _ = fs_ops::write_meta(&path, &meta);
                    let _ = fs_ops::rebuild_manifest(project_root);
                }
            }

            TreeAction::StartRename(path, current_name) => {
                self.renaming_path = Some(path);
                self.rename_draft = current_name;
            }
            TreeAction::CancelRename => {
                self.renaming_path = None;
                self.rename_draft.clear();
            }
            TreeAction::CommitRename(old_path) => {
                let new_name = self.rename_draft.trim().to_string();
                self.renaming_path = None;
                let old_name = old_path.rsplit('/').next().unwrap_or_default();
                if new_name.is_empty() || new_name == old_name {
                    return;
                }
                let split = old_path.rfind('/').unwrap_or(0);
                let dir = old_path[..split].to_string();
                let new_path = format!("{dir}/{new_name}");
                if fs_ops::rename_path(&old_path, &new_path).is_ok() {
                    let _ = fs_ops::rename_path(&format!("{old_path}.meta"), &format!("{new_path}.meta"));
                    if self.selected_path.as_deref() == Some(old_path.as_str()) {
                        self.selected_path = Some(new_path);
                    }
                    self.refresh_dir(&dir);
                    let _ = fs_ops::rebuild_manifest(project_root);
                }
            }

            TreeAction::RequestDelete(path) => self.confirming_delete_path = Some(path),
            TreeAction::CancelDelete => self.confirming_delete_path = None,
            TreeAction::ConfirmDelete(path) => {
                self.confirming_delete_path = None;
                if fs_ops::delete_path(&path).is_ok() {
                    let _ = fs_ops::delete_path(&format!("{path}.meta"));
                    if self.selected_path.as_deref() == Some(path.as_str()) {
                        self.selected_path = None;
                    }
                    let dir = path[..path.rfind('/').unwrap_or(0)].to_string();
                    self.refresh_dir(&dir);
                    let _ = fs_ops::rebuild_manifest(project_root);
                }
            }

            TreeAction::DragOver(target) => self.drop_target = target,

            TreeAction::Drop { source, target } => {
                self.drop_target = None;
                let src_dir = source[..source.rfind('/').unwrap_or(0)].to_string();
                // Also covers "dropped it back into the folder it's already
                // in" — without this, `resolve_conflict` sees the item's
                // own current path as an "existing" file at the target and
                // renames it out from under itself with a `_1` suffix.
                if source == target || src_dir == target || target.starts_with(&format!("{source}/")) {
                    return;
                }
                let name = source.rsplit('/').next().unwrap_or_default().to_string();
                let dst = fs_ops::resolve_conflict(&target, &name);
                if fs_ops::move_path(&source, &dst).is_ok() {
                    let _ = fs_ops::move_path(&format!("{source}.meta"), &format!("{dst}.meta"));
                    self.refresh_dir(&target);
                    self.refresh_dir(&src_dir);
                    let _ = fs_ops::rebuild_manifest(project_root);
                }
            }

            TreeAction::Import(target_dir) => {
                if self.pending_import.is_none() {
                    let (tx, rx) = std::sync::mpsc::channel();
                    std::thread::spawn(move || {
                        let _ = tx.send(fs_ops::pick_file());
                    });
                    self.pending_import = Some((target_dir, rx));
                }
            }

            TreeAction::NewFolder(target_dir) => {
                let dst = fs_ops::resolve_conflict(&target_dir, "new_folder");
                if fs_ops::create_folder(&dst).is_ok() {
                    self.refresh_dir(&target_dir);
                }
            }

            TreeAction::NewComp(target_dir) => {
                let dst = fs_ops::resolve_conflict(&target_dir, "new_prefab.comp");
                if fs_ops::create_comp_file(&dst).is_ok() {
                    fs_ops::get_or_create_meta(&dst);
                    self.refresh_dir(&target_dir);
                    let _ = fs_ops::rebuild_manifest(project_root);
                }
            }
        }
    }
}

/// Actions collected while walking the tree this frame, applied afterward.
/// Keeping this as data (rather than mutating `AssetState` mid-traversal)
/// sidesteps borrow-checker fights between "read the tree to draw it" and
/// "mutate the tree because a row was clicked" within the same pass.
pub enum TreeAction {
    ToggleExpand(String),
    Select(String),
    EnsureMeta(String),
    ToggleIncluded(String),

    StartRename(String, String),
    CancelRename,
    CommitRename(String),

    RequestDelete(String),
    CancelDelete,
    ConfirmDelete(String),

    // Drag-and-drop uses egui's own `DragAndDrop` plugin (see
    // `Response::dnd_set_drag_payload`/`dnd_hover_payload`/
    // `dnd_release_payload` in asset_tree.rs) rather than hand-rolled
    // start/end actions — it correctly tracks "what's under the pointer"
    // via `contains_pointer()`, which (unlike `hovered()`) stays accurate
    // for *other* widgets while a drag is in progress. `DragOver` is still
    // an action purely for the row-highlight/status-bar caching describe
    // in `AssetState::drop_target`'s doc comment; `Drop` carries the
    // dragged item's path directly from the payload, rather than reading
    // back some separately-tracked "currently dragged" state.
    DragOver(Option<String>),
    Drop { source: String, target: String },

    // Triggered from a row's right-click menu now rather than a toolbar
    // button — the target dir comes straight from which row was
    // right-clicked (its own path if a folder, its parent if a file) rather
    // than a separately-tracked "focused" directory.
    Import(String),
    NewFolder(String),
    NewComp(String),
}
