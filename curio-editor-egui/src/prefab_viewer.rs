//! 3D scene preview for a resolved prefab — the Rust equivalent of
//! `PrefabViewport.tsx`.
//!
//! **Move/rotate/scale gizmo:** implemented, but as a 2D screen-space
//! overlay in `prefab_gizmo.rs` (drawn/hit-tested via `ui.painter()` off
//! this module's `view_proj`/camera state), not real 3D geometry inside
//! this file's wgpu pipeline. See `prefab_gizmo.rs`'s doc comment for the
//! reasoning and the specific simplifications that come with that choice
//! (e.g. handles aren't depth-tested against the scene, Euler-additive
//! rotation rather than quaternion composition).
//!
//! **Remaining scope cuts, clearly called out:**
//! - **`RendererDynamic` (Spine `.anim`) entries render as a placeholder
//!   marker**, not the actual animated skeleton. Getting real Spine
//!   rendering into this same 3D scene means transforming 2D skeleton
//!   vertices into a scene's arbitrary world matrix and sharing a draw
//!   pass with the GLB pipeline below — a meaningfully separate
//!   integration task from the single-file, orthographic-camera
//!   `anim_viewer.rs` preview. The marker at least shows *where* the
//!   object sits in the composition.
//! - **Click-to-select is real** (ray-vs-triangle picking against loaded
//!   GLB geometry, brute-force — fine for preview-scene triangle counts),
//!   but selecting a Spine marker isn't implemented (no triangles to hit
//!   test against) — select it from the inspector tree instead.
//!
//! Geometry loading intentionally duplicates `glb_viewer.rs`'s local-space
//! mesh parsing (`collect_geometry`) rather than sharing it — the shapes
//! that library code needs to return differ just enough (this one keeps
//! per-instance world matrices separate from mesh data, for caching sanity
//! when multiple entries reference the same `.glb`) that extracting a
//! common function felt like more risk to the working GLB preview than it
//! was worth for this pass.

use eframe::egui;
use egui_wgpu::wgpu::{self, util::DeviceExt};
use glam::{Mat4, Vec3};
use std::collections::HashMap;
use std::sync::Arc;

use crate::prefab_transforms::{RenderEntry, RendererKind};

// ─────────────────────────────────────────────────────────────────────────────
// Geometry (local-space, per source .glb file — cached across rebuilds)
// ─────────────────────────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}

struct LocalMesh {
    vertices: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    indices: Vec<u32>,
}

fn load_local_mesh(bytes: &[u8]) -> Result<LocalMesh, String> {
    let (document, buffers, _images) = gltf::import_slice(bytes).map_err(|e| format!("Failed to parse GLB: {e}"))?;

    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    fn walk(node: gltf::Node, parent: Mat4, buffers: &[gltf::buffer::Data], vertices: &mut Vec<[f32; 3]>, normals: &mut Vec<[f32; 3]>, indices: &mut Vec<u32>) {
        let local = Mat4::from_cols_array_2d(&node.transform().matrix());
        let world = parent * local;

        if let Some(mesh) = node.mesh() {
            for primitive in mesh.primitives() {
                if primitive.mode() != gltf::mesh::Mode::Triangles {
                    continue;
                }
                let reader = primitive.reader(|b| Some(&buffers[b.index()]));
                let Some(positions) = reader.read_positions() else { continue };
                let positions: Vec<[f32; 3]> = positions.collect();
                let raw_normals: Vec<[f32; 3]> = reader.read_normals().map(|it| it.collect()).unwrap_or_default();

                let base = vertices.len() as u32;
                for (i, p) in positions.iter().enumerate() {
                    let wp = world.transform_point3(Vec3::from(*p));
                    let n = raw_normals.get(i).copied().unwrap_or([0.0, 1.0, 0.0]);
                    let wn = world.transform_vector3(Vec3::from(n)).normalize_or_zero();
                    vertices.push(wp.into());
                    normals.push(wn.into());
                }
                if let Some(read_indices) = reader.read_indices() {
                    indices.extend(read_indices.into_u32().map(|i| base + i));
                } else {
                    indices.extend((0..positions.len() as u32).map(|i| base + i));
                }
            }
        }
        for child in node.children() {
            walk(child, world, buffers, vertices, normals, indices);
        }
    }

    for scene in document.scenes() {
        for node in scene.nodes() {
            walk(node, Mat4::IDENTITY, &buffers, &mut vertices, &mut normals, &mut indices);
        }
    }

    Ok(LocalMesh { vertices, normals, indices })
}

// ─────────────────────────────────────────────────────────────────────────────
// Camera (same orbit design as glb_viewer.rs — small enough not to bother
// sharing)
// ─────────────────────────────────────────────────────────────────────────────

struct OrbitCamera {
    yaw: f32,
    pitch: f32,
    distance: f32,
    target: Vec3,
}

impl OrbitCamera {
    fn framing(center: Vec3, radius: f32) -> Self {
        Self { yaw: 0.6, pitch: 0.35, distance: radius.max(0.5) * 2.6, target: center }
    }
    fn eye(&self) -> Vec3 {
        let x = self.distance * self.pitch.cos() * self.yaw.sin();
        let y = self.distance * self.pitch.sin();
        let z = self.distance * self.pitch.cos() * self.yaw.cos();
        self.target + Vec3::new(x, y, z)
    }
    fn view(&self) -> Mat4 {
        Mat4::look_at_rh(self.eye(), self.target, Vec3::Y)
    }
    fn proj(&self, aspect: f32) -> Mat4 {
        Mat4::perspective_rh(45f32.to_radians(), aspect, 0.05, self.distance.max(1.0) * 20.0 + 10.0)
    }
    fn view_proj(&self, aspect: f32) -> Mat4 {
        self.proj(aspect) * self.view()
    }
    fn apply_input(&mut self, drag_delta: egui::Vec2, scroll_delta: f32) {
        self.yaw -= drag_delta.x * 0.01;
        self.pitch = (self.pitch + drag_delta.y * 0.01).clamp(-1.5, 1.5);
        self.distance = (self.distance * (1.0 - scroll_delta * 0.001)).max(0.05);
    }
}

/// Thin public wrapper so callers don't need `glam` in scope just to call
/// `apply_input`.
pub struct OrbitCameraHandle(OrbitCamera);
impl OrbitCameraHandle {
    pub fn apply_input(&mut self, drag_delta: egui::Vec2, scroll_delta: f32) {
        self.0.apply_input(drag_delta, scroll_delta);
    }
    pub fn eye(&self) -> Vec3 {
        self.0.eye()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PrefabScene
// ─────────────────────────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_proj: [f32; 16],
    light_dir: [f32; 4],
    base_color: [f32; 4],
}

const SHADER_SRC: &str = r#"
struct Uniforms { view_proj: mat4x4<f32>, light_dir: vec4<f32>, base_color: vec4<f32> };
@group(0) @binding(0) var<uniform> u: Uniforms;

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
};

@vertex
fn vs_main(@location(0) position: vec3<f32>, @location(1) normal: vec3<f32>) -> VertexOut {
    var out: VertexOut;
    out.clip_position = u.view_proj * vec4<f32>(position, 1.0);
    out.world_normal = normal;
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let n = normalize(in.world_normal);
    let diffuse = max(dot(n, normalize(u.light_dir.xyz)), 0.0);
    return vec4<f32>(u.base_color.rgb * (0.25 + diffuse * 0.75), 1.0);
}
"#;

/// One selectable range of triangles in the merged buffer, tagged with the
/// prefab-tree path that produced it — used for CPU-side ray picking.
struct PickRange {
    first_index: u32,
    index_count: u32,
    path: Vec<usize>,
}

pub struct PrefabScene {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,

    mesh_cache: HashMap<String, LocalMesh>,

    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    index_count: u32,
    pick_ranges: Vec<PickRange>,
    // CPU-side copies of the merged geometry, kept alongside the GPU
    // buffers specifically for `pick()` — cheap to keep around at preview
    // scene sizes, avoids a GPU readback for picking.
    cpu_vertices: Vec<Vec3>,
    cpu_indices: Vec<u32>,
    markers: Vec<(Vec3, Vec<usize>, String)>, // RendererDynamic placeholder positions

    uniform_buffer: wgpu::Buffer,

    color_texture: Option<wgpu::Texture>,
    depth_texture: Option<wgpu::Texture>,
    output_size: (u32, u32),
    texture_id: Option<egui::TextureId>,

    pub camera: OrbitCameraHandle,
    entries_signature: String,
}

impl PrefabScene {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("prefab_uniforms"), size: std::mem::size_of::<Uniforms>() as u64, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("prefab_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::VERTEX_FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("prefab_bind_group"), layout: &bind_group_layout, entries: &[wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() }] });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("prefab_shader"), source: wgpu::ShaderSource::Wgsl(SHADER_SRC.into()) });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("prefab_pipeline_layout"), bind_group_layouts: &[&bind_group_layout], push_constant_ranges: &[] });
        let vertex_layout = wgpu::VertexBufferLayout { array_stride: std::mem::size_of::<Vertex>() as u64, step_mode: wgpu::VertexStepMode::Vertex, attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3] };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("prefab_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs_main"), buffers: &[vertex_layout], compilation_options: Default::default() },
            fragment: Some(wgpu::FragmentState { module: &shader, entry_point: Some("fs_main"), targets: &[Some(wgpu::ColorTargetState { format: wgpu::TextureFormat::Rgba8UnormSrgb, blend: Some(wgpu::BlendState::REPLACE), write_mask: wgpu::ColorWrites::ALL })], compilation_options: Default::default() }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, cull_mode: None, ..Default::default() },
            depth_stencil: Some(wgpu::DepthStencilState { format: wgpu::TextureFormat::Depth32Float, depth_write_enabled: true, depth_compare: wgpu::CompareFunction::Less, stencil: Default::default(), bias: Default::default() }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            device,
            queue,
            pipeline,
            bind_group,
            mesh_cache: HashMap::new(),
            vertex_buffer: None,
            index_buffer: None,
            index_count: 0,
            pick_ranges: Vec::new(),
            cpu_vertices: Vec::new(),
            cpu_indices: Vec::new(),
            markers: Vec::new(),
            uniform_buffer,
            color_texture: None,
            depth_texture: None,
            output_size: (0, 0),
            texture_id: None,
            camera: OrbitCameraHandle(OrbitCamera::framing(Vec3::ZERO, 2.0)),
            entries_signature: String::new(),
        }
    }

    /// Rebuilds the merged scene buffer if `entries` describes a different
    /// scene than last time (asset paths, hierarchy, or transforms changed
    /// — unlike the original TS, this doesn't bother distinguishing
    /// "structural" from "transform-only" changes; rebuilding a preview
    /// scene's flat vertex list is cheap enough not to need that
    /// optimization here).
    pub fn sync(&mut self, entries: &[RenderEntry]) {
        let signature = signature_of(entries);
        if signature == self.entries_signature {
            return;
        }
        self.entries_signature = signature;

        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut pick_ranges = Vec::new();
        let mut markers = Vec::new();
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);

        for entry in entries {
            if entry.renderer_kind == RendererKind::Dynamic {
                let pos = entry.world_matrix.transform_point3(Vec3::ZERO);
                min = min.min(pos);
                max = max.max(pos);
                markers.push((pos, entry.path.clone(), entry.name.clone()));
                continue;
            }

            if !self.mesh_cache.contains_key(&entry.asset_abs_path) {
                match std::fs::read(&entry.asset_abs_path).map_err(|e| e.to_string()).and_then(|b| load_local_mesh(&b)) {
                    Ok(mesh) => {
                        self.mesh_cache.insert(entry.asset_abs_path.clone(), mesh);
                    }
                    Err(e) => {
                        eprintln!("[PrefabScene] failed to load {}: {e}", entry.asset_abs_path);
                        continue;
                    }
                }
            }
            let Some(mesh) = self.mesh_cache.get(&entry.asset_abs_path) else { continue };

            let base = vertices.len() as u32;
            let first_index = indices.len() as u32;
            for (i, p) in mesh.vertices.iter().enumerate() {
                let wp = entry.world_matrix.transform_point3(Vec3::from(*p));
                min = min.min(wp);
                max = max.max(wp);
                let n = mesh.normals.get(i).copied().unwrap_or([0.0, 1.0, 0.0]);
                let wn = entry.world_matrix.transform_vector3(Vec3::from(n)).normalize_or_zero();
                vertices.push(Vertex { position: wp.into(), normal: wn.into() });
            }
            indices.extend(mesh.indices.iter().map(|&i| base + i));
            pick_ranges.push(PickRange { first_index, index_count: mesh.indices.len() as u32, path: entry.path.clone() });
        }

        self.cpu_vertices = vertices.iter().map(|v| Vec3::from(v.position)).collect();
        self.cpu_indices = indices.clone();

        self.vertex_buffer = (!vertices.is_empty()).then(|| self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("prefab_vertex_buffer"), contents: bytemuck::cast_slice(&vertices), usage: wgpu::BufferUsages::VERTEX }));
        self.index_buffer = (!indices.is_empty()).then(|| self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("prefab_index_buffer"), contents: bytemuck::cast_slice(&indices), usage: wgpu::BufferUsages::INDEX }));
        self.index_count = indices.len() as u32;
        self.pick_ranges = pick_ranges;
        self.markers = markers;

        if min.x <= max.x {
            let center = (min + max) * 0.5;
            let radius = (max - min).length() * 0.5;
            self.camera = OrbitCameraHandle(OrbitCamera::framing(center, radius));
        }
    }

    /// Re-applies bounding-box framing on demand (e.g. a "Reset camera"
    /// button) without forcing a scene rebuild.
    pub fn reset_camera(&mut self) {
        if self.cpu_vertices.is_empty() {
            return;
        }
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        for v in &self.cpu_vertices {
            min = min.min(*v);
            max = max.max(*v);
        }
        let center = (min + max) * 0.5;
        let radius = (max - min).length() * 0.5;
        self.camera = OrbitCameraHandle(OrbitCamera::framing(center, radius));
    }

    fn ensure_target(&mut self, width: u32, height: u32) -> bool {
        if self.output_size == (width, height) && self.color_texture.is_some() {
            return false;
        }
        self.output_size = (width, height);
        self.color_texture = Some(self.device.create_texture(&wgpu::TextureDescriptor { label: Some("prefab_color"), size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2, format: wgpu::TextureFormat::Rgba8UnormSrgb, usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING, view_formats: &[] }));
        self.depth_texture = Some(self.device.create_texture(&wgpu::TextureDescriptor { label: Some("prefab_depth"), size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2, format: wgpu::TextureFormat::Depth32Float, usage: wgpu::TextureUsages::RENDER_ATTACHMENT, view_formats: &[] }));
        true
    }

    pub fn render(&mut self, render_state: &egui_wgpu::RenderState, width: u32, height: u32) -> (egui::TextureId, egui::Vec2) {
        let width = width.max(1);
        let height = height.max(1);
        let resized = self.ensure_target(width, height);

        let view_proj = self.camera.0.view_proj(width as f32 / height as f32);
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&Uniforms { view_proj: view_proj.to_cols_array(), light_dir: [0.4, 0.8, 0.5, 0.0], base_color: [0.75, 0.76, 0.8, 1.0] }));

        let color_texture = self.color_texture.as_ref().unwrap();
        let depth_texture = self.depth_texture.as_ref().unwrap();
        let color_view = color_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("prefab_encoder") });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("prefab_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: &color_view, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.08, g: 0.08, b: 0.09, a: 1.0 }), store: wgpu::StoreOp::Store } })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment { view: &depth_view, depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }), stencil_ops: None }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            if let (Some(vb), Some(ib)) = (&self.vertex_buffer, &self.index_buffer) {
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &self.bind_group, &[]);
                pass.set_vertex_buffer(0, vb.slice(..));
                pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.index_count, 0, 0..1);
            }
        }
        self.queue.submit(std::iter::once(encoder.finish()));

        let force_new = resized || self.texture_id.is_none();
        let mut renderer = render_state.renderer.write();
        let id = match (self.texture_id, force_new) {
            (Some(id), false) => {
                renderer.update_egui_texture_from_wgpu_texture(&self.device, &color_view, wgpu::FilterMode::Linear, id);
                id
            }
            (Some(id), true) => {
                renderer.free_texture(&id);
                renderer.register_native_texture(&self.device, &color_view, wgpu::FilterMode::Linear)
            }
            (None, _) => renderer.register_native_texture(&self.device, &color_view, wgpu::FilterMode::Linear),
        };
        drop(renderer);
        self.texture_id = Some(id);
        (id, egui::vec2(width as f32, height as f32))
    }

    /// Ray-vs-triangle picking (Möller–Trumbore, brute force over every
    /// triangle) — fine for a preview scene's triangle counts. `origin`/
    /// `dir` are in world space. Returns the path of the nearest hit.
    pub fn pick(&self, origin: Vec3, dir: Vec3) -> Option<Vec<usize>> {
        let mut best_t = f32::MAX;
        let mut best_path: Option<&Vec<usize>> = None;

        for range in &self.pick_ranges {
            let start = range.first_index as usize;
            let end = start + range.index_count as usize;
            let Some(tri_indices) = self.cpu_indices.get(start..end) else { continue };
            for tri in tri_indices.chunks_exact(3) {
                let a = self.cpu_vertices[tri[0] as usize];
                let b = self.cpu_vertices[tri[1] as usize];
                let c = self.cpu_vertices[tri[2] as usize];
                if let Some(t) = ray_triangle(origin, dir, a, b, c) {
                    if t < best_t {
                        best_t = t;
                        best_path = Some(&range.path);
                    }
                }
            }
        }
        best_path.cloned()
    }

    pub fn markers(&self) -> &[(Vec3, Vec<usize>, String)] {
        &self.markers
    }

    pub fn view_proj(&self, aspect: f32) -> Mat4 {
        self.camera.0.view_proj(aspect)
    }
}

fn ray_triangle(origin: Vec3, dir: Vec3, a: Vec3, b: Vec3, c: Vec3) -> Option<f32> {
    const EPS: f32 = 1e-6;
    let edge1 = b - a;
    let edge2 = c - a;
    let h = dir.cross(edge2);
    let det = edge1.dot(h);
    if det.abs() < EPS {
        return None;
    }
    let f = 1.0 / det;
    let s = origin - a;
    let u = f * s.dot(h);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = s.cross(edge1);
    let v = f * dir.dot(q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = f * edge2.dot(q);
    (t > EPS).then_some(t)
}

fn signature_of(entries: &[RenderEntry]) -> String {
    let mut s = String::new();
    for e in entries {
        s.push_str(&format!("{:?}|{}|{:?}\n", e.path, e.asset_abs_path, e.world_matrix.to_cols_array()));
    }
    s
}
