//! Built-in procedural primitives.
//!
//! Generates [`ModelAsset`]s (one mesh + one shared lit material) without any
//! glTF files. Everything is centered at the origin with a unit size:
//! cube edge 1, sphere diameter 1, cylinder diameter 1 and height 1 - so a
//! `Transform3D` scale of N yields a shape N units across.
//!
//! Meshes and materials are created once per (type, resolution) and shared via
//! an internal cache - calling `Primitives::cube(...)` twice returns clones of
//! the same `Arc`. Because tinting happens per draw call (via the camera
//! uniform), the single shared material is safe to reuse across differently
//! tinted entities.
//!
//! Note: building a primitive creates GPU buffers, so the first call must
//! happen after `Services` (and the GPU device) are available - i.e. at
//! runtime, not during static initialization.

use std::collections::HashMap;
use std::f32::consts::{PI, TAU};
use std::sync::{Arc, Mutex, OnceLock};

use curio_core::{ASSET_UID_SHADER_LIT, Assets, Matrix4x4, TextureAsset};
use ext_rendering::{
    Material, Mesh,
    data::{material::ShaderDesc, mesh::Vertex},
};

use crate::ModelAsset;

/// Which built-in shape to generate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    Cube,
    Sphere,
    Cylinder,
}

/// Tessellation level of a generated primitive. Cubes have no tessellation -
/// the resolution is ignored for them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrimitiveResolution {
    /// ~12 segments around. Cheap, for props / far away objects.
    Low,
    /// ~24 segments around. Good default.
    Medium,
    /// ~48 segments around. Smooth silhouettes up close.
    High,
}
impl PrimitiveResolution {
    /// Segments around the circumference (sphere slices / cylinder radial segments).
    fn segments(self) -> usize {
        match self {
            PrimitiveResolution::Low => 12,
            PrimitiveResolution::Medium => 24,
            PrimitiveResolution::High => 48,
        }
    }
    /// Stacks for a sphere from pole to pole.
    fn stacks(self) -> usize {
        match self {
            PrimitiveResolution::Low => 8,
            PrimitiveResolution::Medium => 16,
            PrimitiveResolution::High => 32,
        }
    }
}

pub struct Primitives {}
impl Primitives {
    /// Unit cube (edge 1), centered. Resolution is ignored.
    pub fn cube(resolution: PrimitiveResolution) -> Arc<ModelAsset> {
        Self::get(PrimitiveType::Cube, resolution)
    }
    /// Unit sphere (diameter 1), centered.
    pub fn sphere(resolution: PrimitiveResolution) -> Arc<ModelAsset> {
        Self::get(PrimitiveType::Sphere, resolution)
    }
    /// Unit cylinder (diameter 1, height 1), centered, axis along +Y.
    pub fn cylinder(resolution: PrimitiveResolution) -> Arc<ModelAsset> {
        Self::get(PrimitiveType::Cylinder, resolution)
    }

    fn get(kind: PrimitiveType, resolution: PrimitiveResolution) -> Arc<ModelAsset> {
        // cubes never tessellate, so all resolutions share one entry
        let key = match kind {
            PrimitiveType::Cube => (0u8, 0u8),
            PrimitiveType::Sphere => (1, resolution as u8),
            PrimitiveType::Cylinder => (2, resolution as u8),
        };

        let cache = CACHE.get_or_init(Default::default);
        let mut cache = cache.lock().unwrap();
        if let Some(asset) = cache.get(&key) {
            return asset.clone();
        }

        let mesh = match kind {
            PrimitiveType::Cube => Arc::new(build_cube()),
            PrimitiveType::Sphere => Arc::new(build_sphere(resolution.segments(), resolution.stacks())),
            PrimitiveType::Cylinder => Arc::new(build_cylinder(resolution.segments())),
        };
        let material = primitive_material();
        let asset = Arc::new(ModelAsset::new(vec![mesh], vec![material]));

        cache.insert(key, asset.clone());
        asset
    }
}

// ------------------------------------------------------------------
// internals
// ------------------------------------------------------------------

static CACHE: OnceLock<Mutex<HashMap<(u8, u8), Arc<ModelAsset>>>> = OnceLock::new();
static WHITE_TEXTURE: OnceLock<Arc<TextureAsset>> = OnceLock::new();
static MATERIAL: OnceLock<Arc<Material>> = OnceLock::new();

/// 2x2 solid white texture - primitives are untextured, the shader samples this
/// and multiplies by the per-draw tint.
fn white_texture() -> Arc<TextureAsset> {
    let texture = WHITE_TEXTURE.get_or_init(|| {
        let buffer = [255u8; 2 * 2 * 4];
        Arc::new(TextureAsset::new_from_buffer(Some("primitive_white"), 2, 2, &buffer))
    });
    texture.clone()
}

/// Shared lit material: white diffuse, default (white) tint - tinting happens
/// per draw call so one material serves every primitive entity.
fn primitive_material() -> Arc<Material> {
    let material = MATERIAL.get_or_init(|| {
        let shader = Assets::load_asset::<ShaderDesc>(&ASSET_UID_SHADER_LIT);
        let mut mat = Material::new("primitive", shader, false);
        mat.set_texture_with_label(Some(white_texture()), "diffuse");
        mat.finalize();
        Arc::new(mat)
    });
    material.clone()
}

const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const NO_UV1: [f32; 2] = [0.0, 0.0];

fn push_face(verticies: &mut Vec<Vertex>, indicies: &mut Vec<u32>, corners: [[f32; 3]; 4], normal: [f32; 3]) {
    const UVS: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    let base = verticies.len() as u32;
    for c in 0..4 {
        verticies.push(Vertex::new(corners[c], normal, WHITE, UVS[c], NO_UV1));
    }
    // corners are listed counter-clockwise as seen from outside (front face is CCW)
    indicies.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

/// Unit cube: edge 1, centered, flat face normals.
fn build_cube() -> Mesh {
    let h = 0.5;
    let mut verticies = Vec::with_capacity(24);
    let mut indicies = Vec::with_capacity(36);

    push_face(&mut verticies, &mut indicies, [[-h, -h, h], [h, -h, h], [h, h, h], [-h, h, h]], [0.0, 0.0, 1.0]); // front  (+z)
    push_face(&mut verticies, &mut indicies, [[-h, h, -h], [h, h, -h], [h, -h, -h], [-h, -h, -h]], [0.0, 0.0, -1.0]); // back   (-z)
    push_face(&mut verticies, &mut indicies, [[h, -h, h], [h, h, h], [h, h, -h], [h, -h, -h]], [1.0, 0.0, 0.0]); // right  (+x)
    push_face(&mut verticies, &mut indicies, [[-h, -h, -h], [-h, h, -h], [-h, h, h], [-h, -h, h]], [-1.0, 0.0, 0.0]); // left   (-x)
    push_face(&mut verticies, &mut indicies, [[-h, h, h], [h, h, h], [h, h, -h], [-h, h, -h]], [0.0, 1.0, 0.0]); // top    (+y)
    push_face(&mut verticies, &mut indicies, [[-h, -h, -h], [h, -h, -h], [h, -h, h], [-h, -h, h]], [0.0, -1.0, 0.0]); // bottom (-y)

    Mesh::new("primitive_cube".to_string(), verticies, indicies, Matrix4x4::default())
}

/// UV sphere: diameter 1, centered, smooth normals, poles at +-Y.
fn build_sphere(slices: usize, stacks: usize) -> Mesh {
    let mut verticies = Vec::with_capacity((slices + 1) * (stacks + 1));
    let mut indicies = Vec::with_capacity(slices * (stacks - 1) * 6);
    let stride = slices + 1;

    for i in 0..=stacks {
        let theta = PI * (i as f32) / (stacks as f32); // 0 at +y pole
        let (sin_t, cos_t) = theta.sin_cos();
        for j in 0..=slices {
            let phi = TAU * (j as f32) / (slices as f32);
            let (sin_p, cos_p) = phi.sin_cos();
            let normal = [cos_p * sin_t, cos_t, sin_p * sin_t];
            let position = [normal[0] * 0.5, normal[1] * 0.5, normal[2] * 0.5];
            let uv = [(j as f32) / (slices as f32), (i as f32) / (stacks as f32)];
            verticies.push(Vertex::new(position, normal, WHITE, uv, NO_UV1));
        }
    }

    for i in 0..stacks {
        for j in 0..slices {
            let a = (i * stride + j) as u32; // upper left
            let b = a + stride as u32; // lower left
            // pole rows produce degenerate triangles - skip them
            if i != 0 {
                indicies.extend_from_slice(&[a, b, a + 1]);
            }
            if i != stacks - 1 {
                indicies.extend_from_slice(&[a + 1, b, b + 1]);
            }
        }
    }

    Mesh::new("primitive_sphere".to_string(), verticies, indicies, Matrix4x4::default())
}

/// Cylinder: diameter 1, height 1, centered, axis along +Y, capped.
fn build_cylinder(segments: usize) -> Mesh {
    let mut verticies = Vec::with_capacity(segments * 8);
    let mut indicies = Vec::with_capacity(segments * 12);

    // --- side: interleaved top/bottom rings so the seam can hold two uv values ---
    for j in 0..=segments {
        let phi = TAU * (j as f32) / (segments as f32);
        let (sin_p, cos_p) = phi.sin_cos();
        let normal = [cos_p, 0.0, sin_p];
        let u = (j as f32) / (segments as f32);
        verticies.push(Vertex::new([normal[0] * 0.5, 0.5, normal[2] * 0.5], normal, WHITE, [u, 1.0], NO_UV1));
        verticies.push(Vertex::new([normal[0] * 0.5, -0.5, normal[2] * 0.5], normal, WHITE, [u, 0.0], NO_UV1));
    }
    for j in 0..segments {
        let t0 = (j * 2) as u32;
        let b0 = t0 + 1;
        let t1 = t0 + 2;
        let b1 = t0 + 3;
        indicies.extend_from_slice(&[b0, b1, t1, b0, t1, t0]);
    }

    // --- caps: center vertex + ring, uv is the disc seen from the cap ---
    for (normal, y, reverse) in [([0.0, 1.0, 0.0], 0.5, false), ([0.0, -1.0, 0.0], -0.5, true)] {
        let center_index = verticies.len() as u32;
        verticies.push(Vertex::new([0.0, y, 0.0], normal, WHITE, [0.5, 0.5], NO_UV1));
        let ring_base = verticies.len() as u32;
        for j in 0..=segments {
            let phi = TAU * (j as f32) / (segments as f32);
            let (sin_p, cos_p) = phi.sin_cos();
            let uv = [cos_p * 0.5 + 0.5, sin_p * 0.5 + 0.5];
            verticies.push(Vertex::new([cos_p * 0.5, y, sin_p * 0.5], normal, WHITE, uv, NO_UV1));
        }
        for j in 0..segments as u32 {
            if reverse {
                indicies.extend_from_slice(&[center_index, ring_base + j, ring_base + j + 1]);
            } else {
                indicies.extend_from_slice(&[center_index, ring_base + j + 1, ring_base + j]);
            }
        }
    }

    Mesh::new("primitive_cylinder".to_string(), verticies, indicies, Matrix4x4::default())
}
