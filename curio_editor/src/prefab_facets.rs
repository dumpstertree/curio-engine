//! Bridges `facet.manifest` (`fs_ops::get_facets`) into the
//! `FieldDescriptor` lists the prefab inspector renders fields from, with
//! builtin fallbacks for `Transform2D`/`Transform3D`/`RendererStatic`/
//! `RendererDynamic` when a project's facet.manifest doesn't (yet) declare
//! them explicitly.

use crate::fs_ops;
use crate::prefab_types::{self, FieldDescriptor};

/// Field list for one component type — from facet.manifest if present,
/// falling back to the hardcoded builtin field lists otherwise. Facets are
/// read fresh each call (cheap file I/O — same "just re-read it" pattern
/// used by the asset dropdown reading the manifest on every open).
pub fn component_fields(project_root: &str, kind: &str) -> Vec<FieldDescriptor> {
    let facets = fs_ops::get_facets(project_root);
    if let Some(comp) = facets.manifest.iter().find(|c| c.name == kind) {
        return comp.data.iter().map(|f| FieldDescriptor { name: f.name.clone(), kind: f.data.clone() }).collect();
    }
    prefab_types::builtin_component_fields(kind)
}

/// All known component type names — facet.manifest entries plus the
/// builtins, deduplicated, for the "+ Add Facet" menu.
pub fn all_component_types(project_root: &str) -> Vec<String> {
    let facets = fs_ops::get_facets(project_root);
    let mut names: Vec<String> = facets.manifest.iter().map(|c| c.name.clone()).collect();
    for builtin in ["Transform2D", "Transform3D", "RendererStatic", "RendererDynamic"] {
        if !names.iter().any(|n| n == builtin) {
            names.push(builtin.to_string());
        }
    }
    names.sort();
    names
}
