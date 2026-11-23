//! Port of `prefabResolver.ts`. Resolves a `.comp`'s `base:` inheritance
//! chain (A overrides B overrides C, merged bottom-up) into a
//! `ResolvedGameObject` tree that knows, per field, whether each value was
//! inherited or explicitly overridden by the child — this is what lets the
//! inspector show "—" for inherited-but-unset fields and highlight
//! overrides distinctly (`panels/prefab_tab.rs` does the highlighting;
//! this module is pure data).

use crate::fs_ops;
use crate::prefab_types::{split_field, PrefabComponentRaw, PrefabGameObjectRaw};
use std::collections::HashSet;

// ─────────────────────────────────────────────────────────────────────────────
// Resolved types
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ResolvedField {
    pub raw: String,
    pub overridden: bool,
}

#[derive(Debug, Clone)]
pub struct ResolvedComponent {
    pub kind: String,
    pub fields: Vec<ResolvedField>,
    /// true = not present in base (added by the child, or this is a
    /// from-scratch object with no base at all)
    pub added_by_child: bool,
}

#[derive(Debug, Clone)]
pub struct ResolvedGameObject {
    pub enabled: bool,
    pub name: String,
    pub base: Option<String>,
    pub added_by_child: bool,
    pub components: Vec<ResolvedComponent>,
    pub children: Vec<ResolvedGameObject>,
}

// ─────────────────────────────────────────────────────────────────────────────
// YAML load — tolerant, matches the original TS `normalize()`'s defaults.
// Uses `PrefabGameObjectRaw`'s own `#[serde(default = ...)]` fields for that
// tolerance rather than a hand-rolled normalizer; see prefab_types.rs.
// ─────────────────────────────────────────────────────────────────────────────

pub fn load_raw(bytes: &[u8]) -> Result<PrefabGameObjectRaw, String> {
    serde_yaml::from_slice(bytes).map_err(|e| format!("Failed to parse .comp YAML: {e}"))
}

pub fn dump_raw(node: &PrefabGameObjectRaw) -> Result<String, String> {
    serde_yaml::to_string(node).map_err(|e| e.to_string())
}

/// `id_or_path` is either a numeric manifest ID (new format) or a path
/// relative to the project root (legacy format) — mirrors `loadRaw` in the
/// TS resolver exactly, including the ID→URI lookup through the manifest.
fn load_raw_by_id_or_path(project_root: &str, id_or_path: &str) -> Result<PrefabGameObjectRaw, String> {
    let trimmed = id_or_path.trim();
    let rel_path = if !trimmed.is_empty() && trimmed.chars().all(|c| c.is_ascii_digit()) {
        let entries = fs_ops::read_manifest(project_root);
        entries.into_iter().find(|e| e.id.to_string() == trimmed).map(|e| e.uri).ok_or_else(|| format!("No manifest entry for ID {trimmed}"))?
    } else {
        trimmed.to_string()
    };

    let full_path = format!("{project_root}/{rel_path}");
    let bytes = fs_ops::read_file_bytes(&full_path)?;
    load_raw(&bytes)
}

// ─────────────────────────────────────────────────────────────────────────────
// Node resolution when there's no base to merge against
// ─────────────────────────────────────────────────────────────────────────────
/// field is uniformly "inherited" (`from_child = false`, e.g. a base-only
/// child pulled in unmodified) or uniformly "added" (`from_child = true`,
/// e.g. a from-scratch object, or a child-only addition).
fn resolve_no_base(node: &PrefabGameObjectRaw, from_child: bool) -> ResolvedGameObject {
    ResolvedGameObject {
        enabled: node.enabled,
        name: node.name.clone(),
        base: node.base.clone(),
        added_by_child: from_child,
        components: node.components.iter().map(|c| ResolvedComponent { kind: c.kind.clone(), fields: c.fields.iter().map(|f| ResolvedField { raw: f.clone(), overridden: from_child }).collect(), added_by_child: from_child }).collect(),
        children: node.children.iter().map(|ch| resolve_no_base(ch, from_child)).collect(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Public entry point — resolves the full base chain for one node
// ─────────────────────────────────────────────────────────────────────────────

/// Resolves `node`'s `base:` chain (A→B→C, merged bottom-up: C first, then
/// B overrides C, then A overrides B) into a fully-merged
/// `ResolvedGameObject`. `visited_paths` guards against `base:` cycles —
/// a cycle is treated as if `base` were empty, matching the TS behavior
/// (logs and falls back rather than hanging or erroring the whole load).
pub fn resolve_node(project_root: &str, node: &PrefabGameObjectRaw, visited_paths: &mut HashSet<String>) -> ResolvedGameObject {
    let Some(base_path) = node.base.as_deref().map(str::trim).filter(|s| !s.is_empty()).map(str::to_string) else {
        let mut resolved = resolve_no_base(node, true);
        resolved.children = node.children.iter().map(|ch| resolve_node(project_root, ch, visited_paths)).collect();
        return resolved;
    };

    if visited_paths.contains(&base_path) {
        eprintln!("[PrefabResolver] cycle detected: {base_path} already in chain");
        let mut without_base = node.clone();
        without_base.base = None;
        return resolve_node(project_root, &without_base, visited_paths);
    }

    let base_raw = match load_raw_by_id_or_path(project_root, &base_path) {
        Ok(raw) => raw,
        Err(e) => {
            eprintln!("[PrefabResolver] failed to load base \"{base_path}\": {e}");
            let mut without_base = node.clone();
            without_base.base = None;
            return resolve_node(project_root, &without_base, visited_paths);
        }
    };

    let mut next_visited = visited_paths.clone();
    next_visited.insert(base_path);
    let resolved_base = resolve_node(project_root, &base_raw, &mut next_visited);

    let mut merged = merge_resolved_with_child(&resolved_base, node);

    // Re-resolve any merged child whose *raw* counterpart has its own base
    // (the merge above only combined base+child at this level; a
    // grandchild's own inheritance chain still needs walking).
    let mut children = Vec::with_capacity(merged.children.len());
    for ch in merged.children.drain(..) {
        let raw_child = node.children.iter().find(|c| c.name == ch.name);
        match raw_child {
            Some(rc) if rc.base.is_some() => children.push(resolve_node(project_root, rc, &mut next_visited)),
            _ => children.push(ch),
        }
    }
    merged.children = children;

    merged
}

/// Apply a raw child node's overrides onto an already-resolved base.
fn merge_resolved_with_child(base: &ResolvedGameObject, child: &PrefabGameObjectRaw) -> ResolvedGameObject {
    let mut child_comp_map: Vec<&PrefabComponentRaw> = child.components.iter().collect();

    let mut components: Vec<ResolvedComponent> = base
        .components
        .iter()
        .map(|bc| {
            if let Some(pos) = child_comp_map.iter().position(|cc| cc.kind == bc.kind) {
                let cc = child_comp_map.remove(pos);
                let mut child_field_map: Vec<(String, String)> = cc.fields.iter().map(|f| (split_field(f).0, f.clone())).collect();

                let mut fields: Vec<ResolvedField> = bc
                    .fields
                    .iter()
                    .map(|bf| {
                        let key = split_field(&bf.raw).0;
                        if let Some(pos) = child_field_map.iter().position(|(k, _)| k == &key) {
                            let (_, raw) = child_field_map.remove(pos);
                            ResolvedField { raw, overridden: true }
                        } else {
                            bf.clone()
                        }
                    })
                    .collect();
                for (_, raw) in child_field_map {
                    fields.push(ResolvedField { raw, overridden: true });
                }
                ResolvedComponent { kind: bc.kind.clone(), fields, added_by_child: false }
            } else {
                bc.clone()
            }
        })
        .collect();

    for cc in child_comp_map {
        components.push(ResolvedComponent { kind: cc.kind.clone(), fields: cc.fields.iter().map(|f| ResolvedField { raw: f.clone(), overridden: true }).collect(), added_by_child: true });
    }

    // Children merged by name
    let mut resolved_children: Vec<ResolvedGameObject> = Vec::new();
    for bc in &base.children {
        match child.children.iter().find(|c| c.name == bc.name) {
            Some(cc) => resolved_children.push(merge_resolved_with_child(bc, cc)),
            None => resolved_children.push(bc.clone()),
        }
    }
    for cc in &child.children {
        if !base.children.iter().any(|bc| bc.name == cc.name) {
            resolved_children.push(resolve_no_base(cc, true));
        }
    }

    ResolvedGameObject { enabled: child.enabled, name: child.name.clone(), base: child.base.clone(), added_by_child: false, components, children: resolved_children }
}

// ─────────────────────────────────────────────────────────────────────────────
// ResolvedGameObject -> PrefabGameObjectRaw
// ─────────────────────────────────────────────────────────────────────────────

/// Only the child's own overrides — not the inherited base data. This is
/// what actually gets saved to disk (the whole point of `base:` is to keep
/// `.comp` files small, storing only deltas).
pub fn resolved_to_raw(node: &ResolvedGameObject) -> PrefabGameObjectRaw {
    PrefabGameObjectRaw {
        enabled: node.enabled,
        name: node.name.clone(),
        base: node.base.clone(),
        components: node
            .components
            .iter()
            .filter(|c| c.added_by_child || c.fields.iter().any(|f| f.overridden))
            .map(|c| PrefabComponentRaw { kind: c.kind.clone(), fields: c.fields.iter().filter(|f| f.overridden).map(|f| f.raw.clone()).collect() })
            .collect(),
        children: node.children.iter().map(resolved_to_raw).collect(),
    }
}

/// Every field (inherited or overridden), fully merged — used by the 3D
/// viewport, which needs the complete picture regardless of override status.
pub fn resolved_to_raw_full(node: &ResolvedGameObject) -> PrefabGameObjectRaw {
    PrefabGameObjectRaw {
        enabled: node.enabled,
        name: node.name.clone(),
        base: node.base.clone(),
        components: node.components.iter().map(|c| PrefabComponentRaw { kind: c.kind.clone(), fields: c.fields.iter().map(|f| f.raw.clone()).collect() }).collect(),
        children: node.children.iter().map(resolved_to_raw_full).collect(),
    }
}
