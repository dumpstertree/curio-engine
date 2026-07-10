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
    pub focused_dir: String,

    pub drag_path: Option<String>,
    pub drop_target: Option<String>,

    pub renaming_path: Option<String>,
    pub rename_draft: String,

    pub confirming_delete_path: Option<String>,
}

impl AssetState {
    pub fn new() -> Self {
        Self {
            roots: Vec::new(),
            roots_loaded: false,
            load_error: None,
            selected_path: None,
            focused_dir: String::new(),
            drag_path: None,
            drop_target: None,
            renaming_path: None,
            rename_draft: String::new(),
            confirming_delete_path: None,
        }
    }

    /// Call once per frame before rendering the tree — (re)loads the root
    /// listing if it hasn't been loaded yet.
    pub fn ensure_roots(&mut self, project_root: &str) {
        if self.roots_loaded {
            return;
        }
        let root = fs_ops::assets_root(project_root);
        self.focused_dir = root.clone();
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

    pub fn apply(&mut self, action: TreeAction, project_root: &str) {
        match action {
            TreeAction::ToggleExpand(path) => {
                for root in &mut self.roots {
                    if let Some(node) = root.find_mut(&path) {
                        if !node.expanded && !node.loaded {
                            node.load_children();
                        }
                        node.expanded = !node.expanded;
                        if node.entry.is_dir {
                            self.focused_dir = node.entry.path.clone();
                        }
                        break;
                    }
                }
            }

            TreeAction::Select(path) => self.selected_path = Some(path),

            TreeAction::SetFocusedDir(path) => self.focused_dir = path,

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

            TreeAction::DragStart(path) => self.drag_path = Some(path),
            TreeAction::DragOver(target) => self.drop_target = target,
            TreeAction::DragEnd => {
                self.drag_path = None;
                self.drop_target = None;
            }
            TreeAction::Drop(target_dir) => {
                self.drop_target = None;
                let Some(drag_path) = self.drag_path.take() else { return };
                let src_dir = drag_path[..drag_path.rfind('/').unwrap_or(0)].to_string();
                // Also covers "dropped it back into the folder it's already
                // in" — without this, `resolve_conflict` sees the item's
                // own current path as an "existing" file at the target and
                // renames it out from under itself with a `_1` suffix.
                if drag_path == target_dir || src_dir == target_dir || target_dir.starts_with(&format!("{drag_path}/")) {
                    return;
                }
                let name = drag_path.rsplit('/').next().unwrap_or_default().to_string();
                let dst = fs_ops::resolve_conflict(&target_dir, &name);
                if fs_ops::move_path(&drag_path, &dst).is_ok() {
                    let _ = fs_ops::move_path(&format!("{drag_path}.meta"), &format!("{dst}.meta"));
                    self.refresh_dir(&target_dir);
                    self.refresh_dir(&src_dir);
                    let _ = fs_ops::rebuild_manifest(project_root);
                }
            }

            TreeAction::Import => {
                let Some(src) = fs_ops::pick_file() else { return };
                let name = src.rsplit('/').next().unwrap_or(&src).to_string();
                let focused_dir = self.focused_dir.clone();
                let dst = fs_ops::resolve_conflict(&focused_dir, &name);
                if fs_ops::copy_file(&src, &dst).is_ok() {
                    fs_ops::get_or_create_meta(&dst);
                    self.refresh_dir(&focused_dir);
                    let _ = fs_ops::rebuild_manifest(project_root);
                }
            }

            TreeAction::NewFolder => {
                let focused_dir = self.focused_dir.clone();
                let dst = fs_ops::resolve_conflict(&focused_dir, "new_folder");
                if fs_ops::create_folder(&dst).is_ok() {
                    self.refresh_dir(&focused_dir);
                }
            }

            TreeAction::NewComp => {
                let focused_dir = self.focused_dir.clone();
                let dst = fs_ops::resolve_conflict(&focused_dir, "new_prefab.comp");
                if fs_ops::create_comp_file(&dst).is_ok() {
                    fs_ops::get_or_create_meta(&dst);
                    self.refresh_dir(&focused_dir);
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
    SetFocusedDir(String),
    EnsureMeta(String),
    ToggleIncluded(String),

    StartRename(String, String),
    CancelRename,
    CommitRename(String),

    RequestDelete(String),
    CancelDelete,
    ConfirmDelete(String),

    DragStart(String),
    DragOver(Option<String>),
    DragEnd,
    Drop(String),

    Import,
    NewFolder,
    NewComp,
}
