//! Port of `prefabTransforms.ts`. Walks a fully-resolved (raw-full)
//! `PrefabGameObjectRaw` tree, composing world matrices down the hierarchy
//! and collecting one `RenderEntry` per `RendererStatic`/`RendererDynamic`
//! component found on an enabled object — this is what the 3D preview
//! (`prefab_viewer.rs`) iterates to know what to draw and where.
//!
//! Also provides `local_matrix`/`world_matrices_for_path`, used by
//! `prefab_gizmo.rs` to know where to draw the move/rotate/scale handles
//! for the currently-selected object and how to convert a gizmo drag (in
//! world space) back into that object's own parent-local transform fields.

use crate::fs_ops;
use crate::prefab_types::{euler_deg_to_quat, is_renderer, is_transform, read_renderer_asset, read_transform_fields, PrefabGameObjectRaw};
use glam::{Mat4, Quat, Vec3};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RendererKind {
    Static,  // .glb
    Dynamic, // .anim (Spine)
}

#[derive(Debug, Clone)]
pub struct RenderEntry {
    pub path: Vec<usize>,
    pub name: String,
    pub world_matrix: Mat4,
    pub renderer_kind: RendererKind,
    pub asset_abs_path: String,
}

/// A node's own local transform matrix (identity if it has no
/// `Transform2D`/`Transform3D` component at all).
pub fn local_matrix(node: &PrefabGameObjectRaw) -> Mat4 {
    let Some(transform_comp) = node.components.iter().find(|c| is_transform(&c.kind)) else {
        return Mat4::IDENTITY;
    };
    let t = read_transform_fields(transform_comp);
    let q = euler_deg_to_quat(t.rotation);
    Mat4::from_scale_rotation_translation(Vec3::new(t.scale.x, t.scale.y, t.scale.z), Quat::from_xyzw(q.x, q.y, q.z, q.w), Vec3::new(t.position.x, t.position.y, t.position.z))
}

/// Returns `(world_matrix_of_target, world_matrix_of_targets_parent)` by
/// walking `root` down `path`. The parent's world matrix is what a gizmo
/// drag's world-space delta needs to be run through (as a vector, not a
/// point — no translation) to get the equivalent delta in the target's own
/// parent-local `position` field.
pub fn world_matrices_for_path(root: &PrefabGameObjectRaw, path: &[usize]) -> (Mat4, Mat4) {
    let mut current = root;
    let mut world = local_matrix(current);
    let mut parent_world = Mat4::IDENTITY;
    for &idx in path {
        parent_world = world;
        current = &current.children[idx];
        world = world * local_matrix(current);
    }
    (world, parent_world)
}

/// Walks the hierarchy collecting renderer entries, resolving asset IDs to
/// absolute paths via the manifest (same ID→URI lookup the asset dropdown
/// and prefab resolver use).
pub fn collect_render_entries(project_root: &str, root: &PrefabGameObjectRaw) -> Vec<RenderEntry> {
    let manifest = fs_ops::read_manifest(project_root);
    let id_to_uri: std::collections::HashMap<String, String> = manifest.into_iter().map(|e| (e.id.to_string(), e.uri)).collect();

    let mut out = Vec::new();
    walk(project_root, root, Mat4::IDENTITY, &[], &id_to_uri, &mut out);
    out
}

fn walk(project_root: &str, node: &PrefabGameObjectRaw, parent_matrix: Mat4, path: &[usize], id_to_uri: &std::collections::HashMap<String, String>, out: &mut Vec<RenderEntry>) {
    let world_matrix = parent_matrix * local_matrix(node);

    if !node.enabled {
        return;
    }

    for comp in &node.components {
        if !is_renderer(&comp.kind) {
            continue;
        }
        let Some(raw_val) = read_renderer_asset(comp) else { continue };
        let trimmed = raw_val.trim();
        if trimmed.is_empty() {
            continue;
        }

        let abs_path = if trimmed.chars().all(|c| c.is_ascii_digit()) {
            let Some(uri) = id_to_uri.get(trimmed) else { continue }; // unknown ID, skip
            format!("{project_root}/{uri}")
        } else {
            // Legacy path format — relative to assets/
            format!("{}/{trimmed}", fs_ops::assets_root(project_root))
        };

        out.push(RenderEntry {
            path: path.to_vec(),
            name: node.name.clone(),
            world_matrix,
            renderer_kind: if comp.kind == "RendererStatic" { RendererKind::Static } else { RendererKind::Dynamic },
            asset_abs_path: abs_path,
        });
    }

    for (i, child) in node.children.iter().enumerate() {
        let mut child_path = path.to_vec();
        child_path.push(i);
        walk(project_root, child, world_matrix, &child_path, id_to_uri, out);
    }
}
