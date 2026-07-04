//! Real GLB preview: parses mesh geometry with the `gltf` crate, uploads it
//! to GPU buffers, and renders it with a small lit pipeline + orbit camera —
//! this is the actual Rust equivalent of `GlbViewport.tsx`'s three.js scene,
//! built from scratch since there was no existing Rust renderer for it.
//!
//! Scope cut, clearly called out: **no materials or textures** — every mesh
//! renders with a single flat lit color (ambient + one directional light).
//! The original three.js viewport presumably respected the GLB's PBR
//! materials/textures; wiring those up is a meaningful further chunk of work
//! (texture loading + sampler setup + a real PBR or even just
//! Blinn-Phong shader) that felt like a separate task from "get a mesh
//! on screen with a working camera" — happy to take it on next.
//!
//! Rendering reuses the same "register a wgpu texture directly with
//! egui_wgpu::Renderer" trick as the live game viewport
//! (`runner/viewport.rs`), so this preview also has zero CPU readback — it's
//! a genuinely separate `TextureId`/`Renderer` registration though, not
//! reusing the game's texture slot.

use eframe::egui;
use egui_wgpu::wgpu::{self, util::DeviceExt};
use glam::{Mat4, Vec3};
use std::sync::Arc;

// ─────────────────────────────────────────────────────────────────────────────
// Geometry
// ─────────────────────────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_proj: [f32; 16],
    light_dir: [f32; 4],
    base_color: [f32; 4],
}

struct Bounds {
    center: Vec3,
    radius: f32,
}

/// Walks the GLB's scene graph, baking each node's world transform straight
/// into its vertices (simplest way to merge an arbitrary node hierarchy into
/// one draw call for a preview — no per-node draw calls, no skinning).
fn collect_geometry(document: &gltf::Document, buffers: &[gltf::buffer::Data]) -> Result<(Vec<Vertex>, Vec<u32>, Bounds), String> {
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut min = Vec3::splat(f32::MAX);
    let mut max = Vec3::splat(f32::MIN);

    fn walk(node: gltf::Node, parent_world: Mat4, buffers: &[gltf::buffer::Data], vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>, min: &mut Vec3, max: &mut Vec3) {
        let local = Mat4::from_cols_array_2d(&node.transform().matrix());
        let world = parent_world * local;

        if let Some(mesh) = node.mesh() {
            for primitive in mesh.primitives() {
                if primitive.mode() != gltf::mesh::Mode::Triangles {
                    continue; // preview only handles triangle meshes
                }
                let reader = primitive.reader(|b| Some(&buffers[b.index()]));
                let Some(positions) = reader.read_positions() else { continue };
                let positions: Vec<[f32; 3]> = positions.collect();
                let normals: Vec<[f32; 3]> = reader.read_normals().map(|it| it.collect()).unwrap_or_default();

                let base_index = vertices.len() as u32;
                for (i, p) in positions.iter().enumerate() {
                    let world_pos = world.transform_point3(Vec3::from(*p));
                    *min = min.min(world_pos);
                    *max = max.max(world_pos);

                    // Flat fallback normal if the mesh didn't supply one —
                    // good enough for a preview shader, not a real fallback
                    // for actual flat-shaded rendering (no per-face normal
                    // recompute here).
                    let local_normal = normals.get(i).copied().unwrap_or([0.0, 1.0, 0.0]);
                    let world_normal = world.transform_vector3(Vec3::from(local_normal)).normalize_or_zero();

                    vertices.push(Vertex { position: world_pos.into(), normal: world_normal.into() });
                }

                if let Some(read_indices) = reader.read_indices() {
                    indices.extend(read_indices.into_u32().map(|i| base_index + i));
                } else {
                    indices.extend((0..positions.len() as u32).map(|i| base_index + i));
                }
            }
        }

        for child in node.children() {
            walk(child, world, buffers, vertices, indices, min, max);
        }
    }

    for scene in document.scenes() {
        for node in scene.nodes() {
            walk(node, Mat4::IDENTITY, buffers, &mut vertices, &mut indices, &mut min, &mut max);
        }
    }

    if vertices.is_empty() {
        return Err("No triangle meshes found in this GLB".to_string());
    }

    let center = (min + max) * 0.5;
    let radius = (max - min).length() * 0.5;
    Ok((vertices, indices, Bounds { center, radius: radius.max(0.01) }))
}

// ─────────────────────────────────────────────────────────────────────────────
// Orbit camera — mirrors the mouse-drag-to-rotate / scroll-to-zoom controls
// the three.js `OrbitControls` gave the original viewport
// ─────────────────────────────────────────────────────────────────────────────

struct OrbitCamera {
    yaw: f32,
    pitch: f32,
    distance: f32,
    target: Vec3,
}

impl OrbitCamera {
    fn framing(bounds: &Bounds) -> Self {
        Self { yaw: 0.6, pitch: 0.35, distance: bounds.radius * 2.6, target: bounds.center }
    }

    fn eye(&self) -> Vec3 {
        let x = self.distance * self.pitch.cos() * self.yaw.sin();
        let y = self.distance * self.pitch.sin();
        let z = self.distance * self.pitch.cos() * self.yaw.cos();
        self.target + Vec3::new(x, y, z)
    }

    fn view_proj(&self, aspect: f32) -> Mat4 {
        let view = Mat4::look_at_rh(self.eye(), self.target, Vec3::Y);
        let proj = Mat4::perspective_rh(45f32.to_radians(), aspect, 0.05, self.distance.max(1.0) * 20.0 + 10.0);
        proj * view
    }

    fn apply_input(&mut self, drag_delta: egui::Vec2, scroll_delta: f32) {
        self.yaw -= drag_delta.x * 0.01;
        self.pitch = (self.pitch + drag_delta.y * 0.01).clamp(-1.5, 1.5);
        self.distance = (self.distance * (1.0 - scroll_delta * 0.001)).max(0.05);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// GlbPreview — owns everything needed to render one loaded GLB, reused
// across frames while the same file stays selected
// ─────────────────────────────────────────────────────────────────────────────

pub struct GlbPreview {
    pub path: String,
    pub mesh_count: usize,
    pub triangle_count: usize,

    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,

    color_texture: Option<wgpu::Texture>,
    depth_texture: Option<wgpu::Texture>,
    output_size: (u32, u32),

    camera: OrbitCamera,
    bounds: Bounds,

    texture_id: Option<egui::TextureId>,
}

const SHADER_SRC: &str = r#"
struct Uniforms {
    view_proj: mat4x4<f32>,
    light_dir: vec4<f32>,
    base_color: vec4<f32>,
};
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
    let ambient = 0.25;
    let lit = ambient + diffuse * 0.75;
    return vec4<f32>(u.base_color.rgb * lit, 1.0);
}
"#;

impl GlbPreview {
    /// Parses `bytes` as a GLB and builds all GPU resources for it. Returns
    /// `Err` with a human-readable message (shown in the preview panel) on
    /// any parse or GPU-setup failure — a malformed/unsupported GLB
    /// shouldn't be able to crash the editor.
    pub fn load(path: String, bytes: &[u8], device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Result<Self, String> {
        let (document, buffers, _images) = gltf::import_slice(bytes).map_err(|e| format!("Failed to parse GLB: {e}"))?;
        let (vertices, indices, bounds) = collect_geometry(&document, &buffers)?;

        let mesh_count = document.meshes().count();
        let triangle_count = indices.len() / 3;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("glb_vertex_buffer"), contents: bytemuck::cast_slice(&vertices), usage: wgpu::BufferUsages::VERTEX });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("glb_index_buffer"), contents: bytemuck::cast_slice(&indices), usage: wgpu::BufferUsages::INDEX });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("glb_uniforms"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("glb_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("glb_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("glb_shader"), source: wgpu::ShaderSource::Wgsl(SHADER_SRC.into()) });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("glb_pipeline_layout"), bind_group_layouts: &[&bind_group_layout], push_constant_ranges: &[] });

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("glb_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs_main"), buffers: &[vertex_layout], compilation_options: Default::default() },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState { format: wgpu::TextureFormat::Rgba8UnormSrgb, blend: Some(wgpu::BlendState::REPLACE), write_mask: wgpu::ColorWrites::ALL })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, cull_mode: None, ..Default::default() },
            depth_stencil: Some(wgpu::DepthStencilState { format: wgpu::TextureFormat::Depth32Float, depth_write_enabled: true, depth_compare: wgpu::CompareFunction::Less, stencil: Default::default(), bias: Default::default() }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let camera = OrbitCamera::framing(&bounds);

        Ok(Self {
            path,
            mesh_count,
            triangle_count,
            device,
            queue,
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            uniform_buffer,
            bind_group,
            pipeline,
            color_texture: None,
            depth_texture: None,
            output_size: (0, 0),
            camera,
            bounds,
            texture_id: None,
        })
    }

    fn ensure_targets(&mut self, width: u32, height: u32) {
        if self.output_size == (width, height) && self.color_texture.is_some() {
            return;
        }
        self.output_size = (width, height);

        self.color_texture = Some(self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("glb_preview_color"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        }));

        self.depth_texture = Some(self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("glb_preview_depth"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        }));
    }

    /// Renders one frame at `width`x`height`, registers/updates the result
    /// with `render_state`'s shared `egui_wgpu::Renderer`, and returns the
    /// `TextureId` + size to display via `ui.image(...)`. Call this every
    /// frame the preview is visible (camera drag needs continuous repaint
    /// anyway) — `ensure_targets` only actually reallocates on a real size
    /// change.
    pub fn render(&mut self, render_state: &egui_wgpu::RenderState, width: u32, height: u32) -> (egui::TextureId, egui::Vec2) {
        let width = width.max(1);
        let height = height.max(1);
        let resized = self.output_size != (width, height);
        self.ensure_targets(width, height);

        let view_proj = self.camera.view_proj(width as f32 / height as f32);
        let uniforms = Uniforms {
            view_proj: view_proj.to_cols_array(),
            light_dir: [0.4, 0.8, 0.5, 0.0],
            base_color: [0.75, 0.76, 0.8, 1.0],
        };
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        let color_texture = self.color_texture.as_ref().unwrap();
        let depth_texture = self.depth_texture.as_ref().unwrap();
        let color_view = color_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("glb_preview_encoder") });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("glb_preview_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &color_view,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.11, g: 0.11, b: 0.12, a: 1.0 }), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment { view: &depth_view, depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }), stencil_ops: None }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..self.index_count, 0, 0..1);
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

    pub fn handle_input(&mut self, drag_delta: egui::Vec2, scroll_delta: f32) {
        self.camera.apply_input(drag_delta, scroll_delta);
    }

    pub fn reset_camera(&mut self) {
        self.camera = OrbitCamera::framing(&self.bounds);
    }
}
