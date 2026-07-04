//! Port of `prefabTypes.ts`. Pure data + helpers, no I/O — the raw
//! `.comp` shape and the string-encoded field convention
//! (`"position: (0.0,0.0,0.0)"`) are ported field-for-field, byte-for-byte
//! compatible with what the original TS wrote to disk (and what
//! `curio_core`'s `PrefabGameObject`/`PrefabComponent` Rust structs
//! presumably deserialize — this is the on-disk format, not a new one).

use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────────────
// Raw YAML shape
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrefabComponentRaw {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrefabGameObjectRaw {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_name")]
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base: Option<String>,
    #[serde(default)]
    pub components: Vec<PrefabComponentRaw>,
    #[serde(default)]
    pub children: Vec<PrefabGameObjectRaw>,
}

fn default_true() -> bool {
    true
}
fn default_name() -> String {
    "GameObject".to_string()
}

// ─────────────────────────────────────────────────────────────────────────────
// Vectors
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

/// Split a raw field string `"key: value"` / `"key:value"` into (key, value).
pub fn split_field(field: &str) -> (String, String) {
    match field.find(':') {
        Some(idx) => (field[..idx].trim().to_string(), field[idx + 1..].trim().to_string()),
        None => (field.trim().to_string(), String::new()),
    }
}

pub fn join_field(key: &str, value: &str) -> String {
    format!("{key}: {value}")
}

/// Parse `"(1.0,2,3)"` -> `[1.0, 2.0, 3.0]`. Tolerates whitespace and missing parens.
pub fn parse_tuple(value: &str) -> Vec<f32> {
    let inner = value.trim().trim_start_matches('(').trim_end_matches(')');
    if inner.is_empty() {
        return Vec::new();
    }
    inner.split(',').map(|s| s.trim().parse::<f32>().unwrap_or(0.0)).collect()
}

/// Formats a float the way Rust's `f32` Display formatting tends to look —
/// at least one decimal place, so it round-trips as a float on re-parse.
fn format_num(n: f32) -> String {
    if n.fract() == 0.0 {
        format!("{n:.1}")
    } else {
        let mut s = format!("{n:.6}");
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.push('0');
        }
        s
    }
}

pub fn format_tuple(nums: &[f32]) -> String {
    format!("({})", nums.iter().map(|n| format_num(*n)).collect::<Vec<_>>().join(","))
}

pub fn parse_vec2(value: &str) -> Vec2 {
    let t = parse_tuple(value);
    Vec2 { x: t.first().copied().unwrap_or(0.0), y: t.get(1).copied().unwrap_or(0.0) }
}
pub fn parse_vec3(value: &str) -> Vec3 {
    let t = parse_tuple(value);
    Vec3 { x: t.first().copied().unwrap_or(0.0), y: t.get(1).copied().unwrap_or(0.0), z: t.get(2).copied().unwrap_or(0.0) }
}
pub fn format_vec2(v: Vec2) -> String {
    format_tuple(&[v.x, v.y])
}
pub fn format_vec3(v: Vec3) -> String {
    format_tuple(&[v.x, v.y, v.z])
}

/// Euler degrees -> quaternion, XYZ intrinsic order (matches the engine's
/// convention per the original TS comment: "roll, pitch, yaw").
pub fn euler_deg_to_quat(euler: Vec3) -> Quat {
    let to_rad = std::f32::consts::PI / 180.0;
    let x = euler.x * to_rad * 0.5;
    let y = euler.y * to_rad * 0.5;
    let z = euler.z * to_rad * 0.5;
    let (sx, cx) = x.sin_cos();
    let (sy, cy) = y.sin_cos();
    let (sz, cz) = z.sin_cos();

    Quat {
        x: sx * cy * cz + cx * sy * sz,
        y: cx * sy * cz - sx * cy * sz,
        z: cx * cy * sz + sx * sy * cz,
        w: cx * cy * cz - sx * sy * sz,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Component type registry — populated from facet.manifest via
// `prefab_facets.rs`. Reuses `fs_ops::EntryType` rather than duplicating
// the same enum a second time here.
// ─────────────────────────────────────────────────────────────────────────────

pub use crate::fs_ops::EntryType;

#[derive(Debug, Clone)]
pub struct FieldDescriptor {
    pub name: String,
    pub kind: EntryType,
}

pub fn is_transform(kind: &str) -> bool {
    kind == "Transform2D" || kind == "Transform3D"
}
pub fn is_renderer(kind: &str) -> bool {
    kind == "RendererStatic" || kind == "RendererDynamic"
}

/// Fallback field list for Transform/Renderer components not (yet) listed
/// in facet.manifest — mirrors `BUILTIN_COMPONENT_FIELDS`.
pub fn builtin_component_fields(kind: &str) -> Vec<FieldDescriptor> {
    match kind {
        "Transform2D" | "Transform3D" => vec![
            FieldDescriptor { name: "position".to_string(), kind: EntryType::Vector3 },
            FieldDescriptor { name: "rotation".to_string(), kind: EntryType::Vector3 },
            FieldDescriptor { name: "scale".to_string(), kind: EntryType::Vector3 },
        ],
        "RendererStatic" => vec![FieldDescriptor { name: "asset".to_string(), kind: EntryType::Asset(".glb".to_string()) }],
        "RendererDynamic" => vec![FieldDescriptor { name: "asset".to_string(), kind: EntryType::Asset(".anim".to_string()) }],
        _ => Vec::new(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Transform field read/write
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Default)]
pub struct TransformFields {
    pub position: Vec3,
    pub rotation: Vec3, // euler degrees, always 3 components even for Transform2D
    pub scale: Vec3,
}

/// Reads position/rotation/scale out of a Transform2D/Transform3D
/// component's fields, applying engine defaults when a field is absent:
/// position (0,0,0), rotation (0,0,0), scale (1,1,1).
pub fn read_transform_fields(comp: &PrefabComponentRaw) -> TransformFields {
    let is2d = comp.kind == "Transform2D";
    let mut position = Vec3::default();
    let mut rotation = Vec3::default();
    let mut scale = Vec3 { x: 1.0, y: 1.0, z: 1.0 };

    for f in &comp.fields {
        let (key, val) = split_field(f);
        match key.as_str() {
            "position" => {
                position = if is2d {
                    let v = parse_vec2(&val);
                    Vec3 { x: v.x, y: v.y, z: 0.0 }
                } else {
                    parse_vec3(&val)
                };
            }
            "rotation" => rotation = parse_vec3(&val),
            "scale" => {
                scale = if is2d {
                    let t = parse_tuple(&val);
                    Vec3 { x: t.first().copied().unwrap_or(1.0), y: t.get(1).copied().unwrap_or(1.0), z: t.get(2).copied().unwrap_or(1.0) }
                } else {
                    parse_vec3(&val)
                };
            }
            _ => {}
        }
    }
    TransformFields { position, rotation, scale }
}

/// Writes position/rotation/scale back into the component's fields,
/// preserving 2D vs 3D tuple arity and leaving any other fields untouched.
pub fn write_transform_fields(comp: &PrefabComponentRaw, t: TransformFields) -> PrefabComponentRaw {
    let is2d = comp.kind == "Transform2D";
    let mut fields = comp.fields.clone();

    let pos_str = if is2d { format_tuple(&[t.position.x, t.position.y]) } else { format_vec3(t.position) };
    let rot_str = format_vec3(t.rotation);
    let scale_str = if is2d { format_tuple(&[t.scale.x, t.scale.y, t.scale.z]) } else { format_vec3(t.scale) };

    let mut set_or_append = |key: &str, value_str: String| {
        let entry = join_field(key, &value_str);
        if let Some(idx) = fields.iter().position(|f| split_field(f).0 == key) {
            fields[idx] = entry;
        } else {
            fields.push(entry);
        }
    };
    set_or_append("position", pos_str);
    set_or_append("rotation", rot_str);
    set_or_append("scale", scale_str);

    PrefabComponentRaw { kind: comp.kind.clone(), fields }
}

pub fn read_renderer_asset(comp: &PrefabComponentRaw) -> Option<String> {
    comp.fields.iter().find_map(|f| {
        let (key, val) = split_field(f);
        (key == "asset").then_some(val)
    })
}

pub fn write_renderer_asset(comp: &PrefabComponentRaw, asset_path: &str) -> PrefabComponentRaw {
    let mut fields = comp.fields.clone();
    let entry = join_field("asset", asset_path);
    if let Some(idx) = fields.iter().position(|f| split_field(f).0 == "asset") {
        fields[idx] = entry;
    } else {
        fields.push(entry);
    }
    PrefabComponentRaw { kind: comp.kind.clone(), fields }
}

// ─────────────────────────────────────────────────────────────────────────────
// Default factories
// ─────────────────────────────────────────────────────────────────────────────

/// A freshly-added component starts with NO fields set.
pub fn default_component(kind: &str) -> PrefabComponentRaw {
    PrefabComponentRaw { kind: kind.to_string(), fields: Vec::new() }
}

pub fn default_game_object(name: &str) -> PrefabGameObjectRaw {
    PrefabGameObjectRaw { enabled: true, name: name.to_string(), base: None, components: Vec::new(), children: Vec::new() }
}

// ─────────────────────────────────────────────────────────────────────────────
// Raw tree navigation
// ─────────────────────────────────────────────────────────────────────────────

/// Walk a path of child indices down the raw tree.
pub fn get_node_at_path<'a>(root: &'a PrefabGameObjectRaw, path: &[usize]) -> Option<&'a PrefabGameObjectRaw> {
    let mut node = root;
    for &idx in path {
        node = node.children.get(idx)?;
    }
    Some(node)
}

pub fn get_node_at_path_mut<'a>(root: &'a mut PrefabGameObjectRaw, path: &[usize]) -> Option<&'a mut PrefabGameObjectRaw> {
    let mut node = root;
    for &idx in path {
        node = node.children.get_mut(idx)?;
    }
    Some(node)
}

/// Return a new root with the node at `path` replaced by `next`.
pub fn set_node_at_path(root: &PrefabGameObjectRaw, path: &[usize], next: PrefabGameObjectRaw) -> PrefabGameObjectRaw {
    if path.is_empty() {
        return next;
    }
    let mut new_root = root.clone();
    if let Some(target) = get_node_at_path_mut(&mut new_root, path) {
        *target = next;
    }
    new_root
}

/// Add or replace a field on a component by key.
pub fn set_component_field(comp: &PrefabComponentRaw, key: &str, value: &str) -> PrefabComponentRaw {
    let mut fields: Vec<String> = comp.fields.iter().filter(|f| split_field(f).0 != key).cloned().collect();
    fields.push(join_field(key, value));
    PrefabComponentRaw { kind: comp.kind.clone(), fields }
}
