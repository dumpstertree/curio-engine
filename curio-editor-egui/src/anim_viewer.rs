//! Real `.anim` (Spine 3.8) preview.
//!
//! The `.anim` file format was invisible from the uploaded project until
//! `AnimViewport.tsx` was checked directly for this feature — it turns out
//! to be a plain zip containing three files: `skeleton.atlas`,
//! `skeleton.json` (Spine's "3.8 native" JSON export — no version-shim
//! needed), and `skeleton.png`. That maps directly onto `rusty_spine`.
//!
//! Scope cuts, clearly called out:
//! - **Single atlas page only.** The `.anim` format always bundles exactly
//!   one `skeleton.png`, so rather than wiring up `rusty_spine`'s
//!   process-global texture-callback registry (`set_create_texture_cb`/
//!   `set_dispose_texture_cb`, meant for multi-page atlases with per-page
//!   filtering/wrap settings), this just builds one wgpu texture from that
//!   PNG directly and binds it for every draw call, ignoring
//!   `Renderable::attachment_renderer_object` entirely. Correct for every
//!   `.anim` file this format can produce; would misrender a hand-built
//!   multi-page Spine atlas, which isn't a shape this pipeline emits.
//! - **No premultiplied-alpha detection, no per-renderable blend modes.**
//!   Assumes straight alpha + `BlendMode::Normal` for everything. Spine
//!   supports additive/multiply/screen blend modes per slot and a
//!   premultiplied-alpha export option (see `atlas.pages().any(|p|
//!   p.pma())`); a faithful renderer would need a small pipeline-per-blend-
//!   state cache like the `rusty_spine` miniquad example does. Skipped here
//!   for the same reason GLB skipped materials — meaningfully separate
//!   piece of work from "get the skeleton animating on screen."
//! - **No dark-color tinting.** Vertex color only.
//!
//! Confirmed against real compiler errors (this fork's actual API, now
//! verified rather than guessed):
//! - No `Physics` enum and no physics parameter on `update()` — that's a
//!   Spine 4.2-era addition (physics constraints) that postdates the 3.8
//!   runtime this fork targets. `SkeletonController::update` takes just a
//!   delta-time `f32`.
//! - `Renderable::colors` is `Vec<[f32; 4]>` directly, not `Vec<Color>` —
//!   no wrapper struct to unpack.
//! - `SkeletonController::combined_renderables()` takes `&mut self`.
//!
//! Still unverified (no compiler feedback yet on these): `SkeletonData`'s
//! animation/bone/slot accessor names, `Renderable::vertices`/`uvs`/
//! `indices`' exact element types, and `TrackEntry`'s playback accessors
//! (`track_at_index`, `.animation()`, `.duration()`, `.track_time()`) —
//! named to mirror the original TS viewport's `track.animation.duration` /
//! `track.trackTime`. If any of these are wrong, it should be an isolated,
//! obvious fix in `AnimPreview::load`/`AnimPreview::render` — nothing here
//! is deeply threaded through the rest of the editor.

use eframe::egui;
use egui_wgpu::wgpu::{self, util::DeviceExt};
use glam::Mat4;
use rusty_spine::{
    controller::{SkeletonController, SkeletonControllerSettings},
    draw::{ColorSpace, CullDirection},
    AnimationStateData, Atlas, SkeletonJson,
};
use std::sync::{Arc, Once};

// ─────────────────────────────────────────────────────────────────────────────
// Vertex / uniforms
// ─────────────────────────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
    color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_proj: [f32; 16],
}

const SHADER_SRC: &str = r#"
struct Uniforms { view_proj: mat4x4<f32> };
@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var tex: texture_2d<f32>;
@group(0) @binding(2) var samp: sampler;

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(@location(0) position: vec2<f32>, @location(1) uv: vec2<f32>, @location(2) color: vec4<f32>) -> VertexOut {
    var out: VertexOut;
    out.clip_position = u.view_proj * vec4<f32>(position, 0.0, 1.0);
    out.uv = uv;
    out.color = color;
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let tex_color = textureSample(tex, samp, in.uv);
    return tex_color * in.color;
}
"#;

// ─────────────────────────────────────────────────────────────────────────────
// Global texture callbacks — rusty_spine requires these be set process-wide
// before parsing any Atlas. Since we bind our own single texture per
// AnimPreview regardless of what the callback records, these just need to
// exist (a real renderer_object must be present or the C runtime's page
// bookkeeping gets upset) — registered once, ever.
// ─────────────────────────────────────────────────────────────────────────────

static REGISTER_CALLBACKS: Once = Once::new();

/// `RendererObject::set` panics on zero-sized types ("please add member
/// variables") — `()` doesn't qualify, hence this one-byte placeholder.
/// Its value is never read; we bind our own single texture per
/// `AnimPreview` regardless of what's stored here (see module doc).
struct UnusedMarker(u8);

fn ensure_callbacks_registered() {
    REGISTER_CALLBACKS.call_once(|| {
        rusty_spine::extension::set_create_texture_cb(|atlas_page, _path| {
            atlas_page.renderer_object().set(UnusedMarker(0));
        });
        rusty_spine::extension::set_dispose_texture_cb(|atlas_page| unsafe {
            atlas_page.renderer_object().dispose::<UnusedMarker>();
        });
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Zip unpacking — mirrors AnimViewport.tsx's JSZip usage exactly: find
// entries by suffix, error listing what *was* found if a required file is
// missing.
// ─────────────────────────────────────────────────────────────────────────────

struct AnimArchive {
    atlas_text: String,
    json_text: String,
    png_bytes: Vec<u8>,
}

fn unpack_anim_zip(bytes: &[u8]) -> Result<AnimArchive, String> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| format!("Failed to open .anim as a zip: {e}"))?;

    let names: Vec<String> = (0..zip.len()).filter_map(|i| zip.by_index(i).ok().map(|f| f.name().to_string())).collect();

    let find_and_read = |zip: &mut zip::ZipArchive<std::io::Cursor<&[u8]>>, suffix: &str| -> Result<Vec<u8>, String> {
        let name = names.iter().find(|n| n.ends_with(suffix)).ok_or_else(|| format!("Missing {suffix} in .anim. Found: {}", names.join(", ")))?.clone();
        let mut file = zip.by_name(&name).map_err(|e| e.to_string())?;
        let mut buf = Vec::new();
        std::io::Read::read_to_end(&mut file, &mut buf).map_err(|e| e.to_string())?;
        Ok(buf)
    };

    let atlas_bytes = find_and_read(&mut zip, "skeleton.atlas")?;
    let json_bytes = find_and_read(&mut zip, "skeleton.json")?;
    let png_bytes = find_and_read(&mut zip, "skeleton.png")?;

    Ok(AnimArchive {
        atlas_text: String::from_utf8(atlas_bytes).map_err(|e| format!("skeleton.atlas is not valid UTF-8: {e}"))?,
        json_text: String::from_utf8(json_bytes).map_err(|e| format!("skeleton.json is not valid UTF-8: {e}"))?,
        png_bytes,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// 2D camera — pan (drag) + zoom (scroll), auto-framed from the skeleton's
// vertex bounds on load
// ─────────────────────────────────────────────────────────────────────────────

struct Camera2D {
    center: glam::Vec2,
    half_height: f32,
}

impl Camera2D {
    fn framing(min: glam::Vec2, max: glam::Vec2) -> Self {
        let center = (min + max) * 0.5;
        let half_height = ((max.y - min.y).max(max.x - min.x) * 0.5 * 1.25).max(10.0);
        Self { center, half_height }
    }

    fn view_proj(&self, aspect: f32) -> Mat4 {
        let half_w = self.half_height * aspect;
        Mat4::orthographic_rh(self.center.x - half_w, self.center.x + half_w, self.center.y - self.half_height, self.center.y + self.half_height, -1000.0, 1000.0)
    }

    fn apply_input(&mut self, drag_delta: egui::Vec2, scroll_delta: f32, aspect: f32) {
        let world_per_px = (self.half_height * 2.0) / (aspect * 800.0).max(1.0); // rough, fine for a preview pan
        self.center.x -= drag_delta.x * world_per_px;
        self.center.y += drag_delta.y * world_per_px;
        self.half_height = (self.half_height * (1.0 - scroll_delta * 0.001)).max(1.0);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AnimPreview
// ─────────────────────────────────────────────────────────────────────────────

pub struct AnimPreview {
    pub path: String,
    pub animations: Vec<String>,
    pub current_animation: String,
    pub bone_count: usize,
    pub slot_count: usize,
    pub duration: f32,
    pub elapsed: f32,

    controller: SkeletonController,
    last_update: std::time::Instant,

    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,

    color_texture: Option<wgpu::Texture>,
    output_size: (u32, u32),
    texture_id: Option<egui::TextureId>,

    camera: Camera2D,
    bounds_min: glam::Vec2,
    bounds_max: glam::Vec2,
}

impl AnimPreview {
    pub fn load(path: String, bytes: &[u8], device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Result<Self, String> {
        ensure_callbacks_registered();

        let archive = unpack_anim_zip(bytes)?;

        let decoded_png = image::load_from_memory(&archive.png_bytes).map_err(|e| format!("Failed to decode skeleton.png: {e}"))?.to_rgba8();
        let (tex_w, tex_h) = decoded_png.dimensions();

        let atlas = Arc::new(Atlas::new(archive.atlas_text.as_bytes(), "").map_err(|e| format!("Failed to parse skeleton.atlas: {e}"))?);
        let skeleton_json = SkeletonJson::new(atlas);
        let skeleton_data = Arc::new(skeleton_json.read_skeleton_data(archive.json_text.as_bytes()).map_err(|e| format!("Failed to parse skeleton.json: {e}"))?);

        let animations: Vec<String> = skeleton_data.animations().map(|a| a.name().to_string()).collect();
        let bone_count = skeleton_data.bones().count();
        let slot_count = skeleton_data.slots().count();

        let animation_state_data = Arc::new(AnimationStateData::new(skeleton_data.clone()));
        let mut controller = SkeletonController::new(skeleton_data, animation_state_data).with_settings(SkeletonControllerSettings { premultiplied_alpha: false, cull_direction: CullDirection::CounterClockwise, color_space: ColorSpace::SRGB });

        let current_animation = animations.first().cloned().unwrap_or_default();
        if !current_animation.is_empty() {
            controller.animation_state.set_animation_by_name(0, &current_animation, true).map_err(|e| format!("Failed to start animation '{current_animation}': {e}"))?;
        }

        // One pose update so there's real vertex data to frame the camera
        // against before the first paint.
        controller.update(0.0);
        let (bounds_min, bounds_max) = compute_bounds(&mut controller);
        let camera = Camera2D::framing(bounds_min, bounds_max);

        // ── GPU setup ────────────────────────────────────────────────────────
        let texture = device.create_texture_with_data(
            &queue,
            &wgpu::TextureDescriptor {
                label: Some("anim_atlas_texture"),
                size: wgpu::Extent3d { width: tex_w, height: tex_h, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            decoded_png.as_raw(),
        );
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor { mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear, address_mode_u: wgpu::AddressMode::ClampToEdge, address_mode_v: wgpu::AddressMode::ClampToEdge, ..Default::default() });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("anim_uniforms"), size: std::mem::size_of::<Uniforms>() as u64, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("anim_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::VERTEX, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("anim_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&texture_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("anim_shader"), source: wgpu::ShaderSource::Wgsl(SHADER_SRC.into()) });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("anim_pipeline_layout"), bind_group_layouts: &[&bind_group_layout], push_constant_ranges: &[] });

        let vertex_layout = wgpu::VertexBufferLayout { array_stride: std::mem::size_of::<Vertex>() as u64, step_mode: wgpu::VertexStepMode::Vertex, attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4] };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("anim_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs_main"), buffers: &[vertex_layout], compilation_options: Default::default() },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState { format: wgpu::TextureFormat::Rgba8UnormSrgb, blend: Some(wgpu::BlendState::ALPHA_BLENDING), write_mask: wgpu::ColorWrites::ALL })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, cull_mode: None, ..Default::default() },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Ok(Self {
            path,
            animations,
            current_animation,
            bone_count,
            slot_count,
            duration: 0.0,
            elapsed: 0.0,
            controller,
            last_update: std::time::Instant::now(),
            device,
            queue,
            pipeline,
            bind_group,
            uniform_buffer,
            color_texture: None,
            output_size: (0, 0),
            texture_id: None,
            camera,
            bounds_min,
            bounds_max,
        })
    }

    pub fn set_animation(&mut self, name: &str) {
        if self.controller.animation_state.set_animation_by_name(0, name, true).is_ok() {
            self.current_animation = name.to_string();
        }
    }

    fn ensure_target(&mut self, width: u32, height: u32) -> bool {
        if self.output_size == (width, height) && self.color_texture.is_some() {
            return false;
        }
        self.output_size = (width, height);
        self.color_texture = Some(self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("anim_preview_color"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        }));
        true
    }

    /// Advances playback, rebuilds the frame's mesh, renders, and
    /// registers/updates the shared egui texture. Call every frame the
    /// preview is visible.
    pub fn render(&mut self, render_state: &egui_wgpu::RenderState, width: u32, height: u32) -> (egui::TextureId, egui::Vec2) {
        let width = width.max(1);
        let height = height.max(1);
        let resized = self.ensure_target(width, height);

        let now = std::time::Instant::now();
        let dt = (now - self.last_update).as_secs_f32().min(0.1);
        self.last_update = now;
        self.controller.update(dt);

        if let Some(track) = self.controller.animation_state.track_at_index(0) {
            self.duration = track.animation().duration();
            self.elapsed = if self.duration > 0.0 { track.track_time() % self.duration } else { 0.0 };
        }

        // Build one merged mesh across all renderables — see module doc for
        // why per-renderable blend-mode/texture handling is skipped.
        let renderables = self.controller.combined_renderables();
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        for renderable in &renderables {
            let base = vertices.len() as u32;
            for i in 0..renderable.vertices.len() {
                let uv = renderable.uvs.get(i).copied().unwrap_or([0.0, 0.0]);
                let c = renderable.colors.get(i).copied().unwrap_or([1.0, 1.0, 1.0, 1.0]);
                vertices.push(Vertex { position: renderable.vertices[i], uv, color: c });
            }
            indices.extend(renderable.indices.iter().map(|&i| base + i as u32));
        }

        let view_proj = self.camera.view_proj(width as f32 / height as f32);
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&Uniforms { view_proj: view_proj.to_cols_array() }));

        let color_texture = self.color_texture.as_ref().unwrap();
        let color_view = color_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("anim_preview_encoder") });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("anim_preview_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: &color_view, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.08, g: 0.08, b: 0.09, a: 1.0 }), store: wgpu::StoreOp::Store } })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if !indices.is_empty() {
                let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("anim_vertex_buffer"), contents: bytemuck::cast_slice(&vertices), usage: wgpu::BufferUsages::VERTEX });
                let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("anim_index_buffer"), contents: bytemuck::cast_slice(&indices), usage: wgpu::BufferUsages::INDEX });

                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &self.bind_group, &[]);
                pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
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

    pub fn handle_input(&mut self, drag_delta: egui::Vec2, scroll_delta: f32, aspect: f32) {
        self.camera.apply_input(drag_delta, scroll_delta, aspect);
    }

    pub fn reset_camera(&mut self) {
        self.camera = Camera2D::framing(self.bounds_min, self.bounds_max);
    }
}

fn compute_bounds(controller: &mut SkeletonController) -> (glam::Vec2, glam::Vec2) {
    let mut min = glam::Vec2::splat(f32::MAX);
    let mut max = glam::Vec2::splat(f32::MIN);
    for renderable in controller.combined_renderables() {
        for v in &renderable.vertices {
            let p = glam::Vec2::from(*v);
            min = min.min(p);
            max = max.max(p);
        }
    }
    if !min.is_finite() || !max.is_finite() {
        min = glam::Vec2::splat(-100.0);
        max = glam::Vec2::splat(100.0);
    }
    (min, max)
}
