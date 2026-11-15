use camera::SysRecordCamera;
use curio_core::{GraphicsMapping, Ledger, Services};
use egui_wgpu::wgpu::{self, BindGroup};
use egui_wgpu::wgpu::util::DeviceExt;
use std::borrow::Cow;
use std::num::NonZeroU64;
use std::sync::Arc;

use cgmath::{Matrix4, SquareMatrix};

use bytemuck::{Pod, Zeroable, cast_slice};

use crate::PostProcessResources;
use crate::record::sys_record_stroke_paint::SysRecordStrokePaint;
use crate::render_feature_post_process::{PostProcessSource, RenderFeaturePostProcess};

const DITHER_COMMON_WGSL: &str = include_str!("../dither_common.wgsl");
const PAINTERLY_WGSL: &str = include_str!("../painterly_strokes.wgsl");

// the hybrid: the previous chain stage (flow-driven kuwahara) is the COLOUR
// GUIDE. pass 1 blits it, pass 2 stamps daubs over it. there is no stroke list
// to keep in sync: slot parameters are regenerated in the vertex stage from an
// R2 low-discrepancy sequence, so the only CPU state is the uniform below
const LAYER_SLOTS: u32 = 4096; // per-layer grid: must match SLOT_COUNT in the WGSL
const STAMP_SLOTS: u32 = 3 * LAYER_SLOTS; // one FULL grid per layer (mass, mid, detail):
//                        the contrast gate picks each region's SCALE, it must
//                        never thin its coverage, so every layer gets the whole
//                        lattice of rays, not a global slot fraction
const STAMP_VERTICES: u32 = 6 * STAMP_SLOTS; // two triangles per slot
const BRUSH_SIZE: u32 = 64; // procedural bristle mask (power of two, rows pad to 4)

// hash for the procedural mask: integer mix, no noise asset needed. wrapping
// arithmetic so a debug build never traps on the overflow
fn mask_hash(x: u32, y: u32, salt: u32) -> f32 {
    let mut v: u32 = x.wrapping_mul(18674711).wrapping_add(y.wrapping_mul(20081977).wrapping_add(salt.wrapping_mul(28714933)));
    v ^= v >> 15;
    v = v.wrapping_mul(2246822519);
    v ^= v >> 13;
    v = v.wrapping_mul(3261131959);
    v ^= v >> 16;
    return (v as f32) / 4294967295.0;
}

fn mask_smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

// one bristle mask for every daub. the stamp fragment samples it in the
// daub's OWN space (u runs along the long axis, v across the short one), so
// the comb bands depend on v only and the bristles trail the stroke instead
// of cutting across it. the rim is noise-cut for a ragged silhouette and the
// body is speckled so a dry drag lets the guide show through, which is what
// real brush alpha buys over an analytic ellipse
fn build_brush_mask() -> Vec<u8> {
    let size = BRUSH_SIZE as f32;
    let half = size * 0.5;
    let mut bytes: Vec<u8> = Vec::new();
    for y in 0..BRUSH_SIZE {
        for x in 0..BRUSH_SIZE {
            // quad [-1,1] with texel centres, matching the shader's local space
            let u = (x as f32 + 0.5 - half) / half;
            let v = (y as f32 + 0.5 - half) / half;
            let r = (u * u + v * v).sqrt();

            // ragged rim: the same noise nudges the silhouette in and out
            let rim = 0.82 + 0.22 * (mask_hash(x, y, 7) - 0.5) * 2.0;
            let mut alpha = 1.0 - mask_smoothstep(rim - 0.42, rim, r);

            // bristle comb: bands stacked along v, each with its own phase and
            // width, so the gaps run the length of the daub
            let band = (v * 0.5 + 0.5) * 11.0;
            let bandId = band.floor() as u32;
            let frac = band - band.floor();
            let width = 0.42 + 0.44 * mask_hash(0, bandId, 11);
            let phase = mask_hash(0, bandId, 3) * 0.45;
            alpha *= 1.0 - mask_smoothstep(width, (width + 0.34).min(1.0), frac + phase);

            // speckle: dry bristle leaves bare tracks
            alpha *= 0.66 + 0.34 * mask_hash(x, y, 5);

            alpha = alpha.clamp(0.0, 1.0);
            bytes.push((alpha * 255.0) as u8);
        }
    }
    return bytes;
}

/// CPU-side mirror of the WGSL StrokeParams struct: 20 scalars, then the 3-vec4
/// layers array at offset 80 and the 4-vec4 vp at 128 exactly as naga lays it
/// out (192 bytes total)
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct StrokeUniform {
    size: f32, // long-axis daub length, px
    size_min: f32, // fraction of size kept where local contrast is maxed
    mag_scale: f32, // slope (luminance per px) * this -> 0..1
    elongation: f32, // aspect where the neighbourhood is anisotropic (a real edge)
    aspect_min: f32, // aspect where there is no edge to follow
    alpha: f32, // stamp opacity
    feather: f32, // 0 = hard-edged stamp, 1 = gradient all the way in
    keep: f32, // keep probability per slot; failures degenerate
    jitter: f32, // anchor jitter as a fraction of one slot cell
    wobble: f32, // angle jitter amplitude in radians
    seed: f32, // shifts every hash
    viewport_count: f32, // number of valid vp rects (1..4)
    flow: f32, // 0 = constant elongation, 1 = shape follows tensor anisotropy
    bristle: f32, // 0 = analytic alpha, 1 = fully masked by the brush texture
    color_jitter: f32, // 0 = exact guide colour, per-stamp hue/lightness shift
    world_size: f32, // 0 = px daubs; > 0 = world-unit long axis through the
    //                     focal length, so distant strokes shrink with the scene
    time: f32, // rolling CPU clock, feeds the world-anchor temporal drift
    world_cell: f32, // world-space lattice spacing the ray hits snap to
    drift: f32, // world-anchor breathing as a fraction of one lattice cell
    life: f32, // birth/death cycle mean in seconds (0 = hard pops kept)
    layers: [[f32; 4]; 3], // per layer (mass, mid, detail): long-axis mult,
    //                        alpha mult, contrast gate lo, gate hi; slots
    //                        [0,4096), [4096,8192), [8192,12288) stamp in this
    //                        order, so mass sits UNDER the fine layer
    vp: [[f32; 4]; 4], // viewport rects in uv: minx, miny, maxx, maxy
}

/// CPU mirror of the WGSL WorldAnchorParams struct (160 bytes): the matrices
/// bind exactly as the 3D pass' CameraUniform does (column arrays, so the WGSL
/// mat4x4 sees the same layout), and the inverse is what unprojects the depth
/// buffer into the world lattice
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Pod, Zeroable)]
pub struct WorldAnchorUniform {
    view_proj: [[f32; 4]; 4],
    inv_view_proj: [[f32; 4]; 4],
    cam_pos: [f32; 4], // world-space camera position (kept for distance fades)
    focal_px: f32, // capture focal length in pixels (y axis)
    pad_x: f32,
    pad_y: f32,
    pad_z: f32,
}

pub struct RenderFeaturePostProcessPainterly {
    blit_pipeline: wgpu::RenderPipeline, // guide -> output, REPLACE
    stamp_pipeline: wgpu::RenderPipeline, // instanced daubs, ALPHA_BLENDING
    bind_group_input: wgpu::BindGroup, // guide = the original offscreen_view
    bind_group_a: wgpu::BindGroup, // guide = view_a
    bind_group_b: wgpu::BindGroup, // guide = view_b
    params: wgpu::Buffer, // StrokeUniform, rewritten per frame
    params_uniform: StrokeUniform,
    world_anchor: wgpu::Buffer, // WorldAnchorUniform, rewritten per frame
    world_anchor_uniform: WorldAnchorUniform,
    stroke_time: f32, // rolling clock, advanced in update_params each frame
    brush_texture: wgpu::Texture, // procedural bristle alpha mask (R8Unorm)
    brush_view: wgpu::TextureView, // bound as binding(3) for the stamp fragment
}

impl RenderFeaturePostProcessPainterly {
    pub fn new(
        device: Arc<wgpu::Device>,
        format: wgpu::TextureFormat,
        post_resources: &PostProcessResources,
        // the stamp pass sobels the GUIDE, never the depth buffer: keeping the
        // pass free of kuwahara's flow textures is what lets the two features
        // own their resources independently. the one exception is the depth
        // TEXTURE itself: the world anchor unprojects the slot ray to the
        // surface it hits, which is what stops strokes sliding over the screen
        depth_view: &wgpu::TextureView,
        offscreen_view: &wgpu::TextureView,
    ) -> Box<Self> {
        let params_uniform = StrokeUniform {
            // reference-scale daubs: long axis 22 px in flat areas, 7-8 px in
            // the busiest ones, held along edges at 2.2:1
            size: 22.0,
            size_min: 0.35,
            mag_scale: 45.0, // SLOPE sensitivity now that dirDeriv divides out the 2r displacement: a 0.4-lumi grout jump over ~15px probe stands at ~0.026 per px, and 45 pins that at full contrast (small round daub), while faint interior crack detail (~0.007 slope) stays near 0.3 for long flat daubs
            elongation: 2.6, // real edges carry the full stretch now
            aspect_min: 1.2, // no edge to follow: near-round daub
            alpha: 1.0,
            feather: 0.45,
            keep: 1.0, // every slot draws: coverage is the first thing to see
            jitter: 0.45,
            wobble: 0.25,
            seed: 0.0,
            viewport_count: 1.0, // whole-texture rect until the helper syncs
            flow: 1.0, // daub SHAPE follows the tensor's anisotropy
            bristle: 0.75, // brush mask drives the stamp alpha
            color_jitter: 0.16, // forced per-stamp deviation from the guide (each channel moves by at least 0.6 * this), which is what makes coverage read as paint
            // daub SCALE stays in px by default: the world anchor only pins
            // WHERE strokes sit; a positive world_size switches the long axis
            // to world units, shrinking distant paint with the scene
            world_size: 0.0,
            time: 0.0, // CPU clock starts here, advanced once per frame
            world_cell: 0.25, // reference-scale daub lattice, world units
            drift: 0.02, // sinusoidal sway of a daub inside its own lattice
            // cell: 2% of a cell is a hair of motion, enough to read as
            // brush breathing rather than as the field sliding
            life: 4.0, // mean birth/death cycle: each stroke fades up and down
            //          on a phase of its own, so camera reveals land mid-fade
            // three contrast-gated layers over one draw: EACH layer gets its own
            // full 4096-ray grid (mass [0,4096), mid [4096,8192), detail the rest)
            // so the gate picks a region's stroke SCALE without ever thinning its
            // coverage; mass only in flat areas, detail only in busy ones, mid in
            // between; alpha mult keeps the masses quiet so they read as an underpainting
            layers: [
                [3.0, 0.45, 0.0, 0.35], // mass: broad low-alpha value masses
                [1.5, 0.8, 0.35, 0.7], // mid-tone strokes over the masses
                [1.0, 1.0, 0.7, 1.0], // detail: the existing daub regime
            ],
            vp: [[0.0, 0.0, 1.0, 1.0], [0.0, 0.0, 1.0, 1.0], [0.0, 0.0, 1.0, 1.0], [0.0, 0.0, 1.0, 1.0]],
        };

        // identity anchor pair until the per-frame camera sync rewrites it: an
        // identity view_proj unprojects the slot ray straight through the
        // depth value, which keeps the construction safe if no camera record
        // is present yet
        let world_anchor_uniform = WorldAnchorUniform {
            view_proj: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            inv_view_proj: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            cam_pos: [0.0, 0.0, 0.0, 1.0],
            focal_px: 1.0,
            pad_x: 0.0,
            pad_y: 0.0,
            pad_z: 0.0,
        };

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("painterly sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // one procedural bristle mask for every daub, uploaded once at
        // construction: the mask is sampled in the DAUB's space, so the
        // streaks trail the stroke instead of cutting across it, and swapping
        // in the game's real brush art later is a texture, not a shader edit
        let brush_size = wgpu::Extent3d {
            width: BRUSH_SIZE,
            height: BRUSH_SIZE,
            depth_or_array_layers: 1,
        };
        let brush_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("painterly brush mask"),
            size: brush_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let brush_bytes = build_brush_mask();
        let s = Services::get();
        s.gpu.queue().write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &brush_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &brush_bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(BRUSH_SIZE), // R8: one byte per texel, and 64 already pads to 4
                rows_per_image: Some(BRUSH_SIZE),
            },
            brush_size,
        );
        let brush_view = brush_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let shader_source = format!("{}\n{}", DITHER_COMMON_WGSL, PAINTERLY_WGSL);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("painterly shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader_source.as_str())),
        });

        let params = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("painterly params"),
            contents: cast_slice(&[params_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let world_anchor = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("painterly world anchor"),
            contents: cast_slice(&[world_anchor_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // one bind layout for both passes: the guide texture is read by the
        // blit FRAGMENT and by the stamp VERTEX stage (its sobel taps), so its
        // visibility spans both
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("painterly bind group layout"),
            entries: &[
                // sampler (blit pass)
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // colour guide (both stages)
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // stamp parameters (the vertex stage derives every daub from
                // them, so it needs the uniform too)
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(192),
                    },
                    count: None,
                },
                // bristle alpha mask (stamp FRAGMENT only)
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // original frame: the stamp VERTEX probes continuous scene
                // gradients for size and orientation; the guide is piecewise
                // constant and nulls those probes over every flat block
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // scene depth: the world anchor unprojects the slot ray to the
                // surface it hits (and re-tests that surface at the projected
                // pixel so occluded lattice points never paint); stamp VERTEX
                // only, so the blit pass never sees it
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // the 3D pass' view_proj plus its CPU inverse: unprojection
                // for the anchor and re-projection to put the daub back
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(160),
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("painterly pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let blit_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("painterly blit pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_fullscreen"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_blit"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // no vertex buffer: the vertex stage derives each daub from its slot
        // index, so the draw is 6*STAMP_SLOTS bare vertices. cull_mode stays
        // None because the uv->ndc row flip reflects every quad, and
        // rejected slots are already zero-area rather than back-facing
        let stamp_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("painterly stamp pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_stamp"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_stamp"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None, // the uv->ndc row flip mirrors every quad
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let bind_group_input = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("painterly bind group INPUT"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(offscreen_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &params,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&brush_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(offscreen_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &world_anchor,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        let bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("painterly bind group A"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&post_resources.view_a),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &params,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&brush_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(offscreen_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &world_anchor,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        let bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("painterly bind group B"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&post_resources.view_b),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &params,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&brush_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(offscreen_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &world_anchor,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        Box::new(Self {
            blit_pipeline,
            stamp_pipeline,
            bind_group_input,
            bind_group_a,
            bind_group_b,
            params,
            params_uniform,
            world_anchor,
            world_anchor_uniform,
            stroke_time: 0.0,
            brush_texture,
            brush_view,
        })
    }

    // rewrite the stamp uniform only when a knob actually moved
    pub fn update(
        &mut self,
        size: f32,
        size_min: f32,
        mag_scale: f32,
        elongation: f32,
        aspect_min: f32,
        alpha: f32,
        feather: f32,
        keep: f32,
        jitter: f32,
        wobble: f32,
        seed: f32,
        flow: f32,
        bristle: f32,
        color_jitter: f32,
        world_size: f32,
        world_cell: f32,
        drift: f32,
        life: f32,
        mass_mult: f32,
        mass_alpha: f32,
        mid_mult: f32,
        mid_alpha: f32,
        fine_mult: f32,
        fine_alpha: f32,
        contrast_split_low: f32,
        contrast_split_high: f32,
    ) {
        // headroom up to 128 px so the stamps can be compared against the
        // reference daub scale without re-touching the WGSL
        let size = size.clamp(1.0, 128.0);
        let size_min = size_min.clamp(0.05, 1.0);
        let mag_scale = mag_scale.max(0.0);
        let elongation = elongation.clamp(1.0, 8.0);
        let aspect_min = aspect_min.clamp(1.0, 8.0);
        let alpha = alpha.clamp(0.0, 1.0);
        let feather = feather.clamp(0.0, 1.0);
        let keep = keep.clamp(0.0, 1.0);
        let jitter = jitter.clamp(0.0, 1.0);
        let wobble = wobble.clamp(0.0, 3.1415927);
        let seed = seed % 1024.0;
        let flow = flow.clamp(0.0, 1.0);
        let bristle = bristle.clamp(0.0, 1.0);
        let color_jitter = color_jitter.clamp(0.0, 0.3);
        // world units: 0 keeps px daubs; 0.25 puts a reference-scale daub
        // roughly a quarter of a metre across
        let world_size = world_size.max(0.0);
        // the lattice spacing the unprojected hit points snap to; never zero
        // or the WGSL divide in floor(world / cell) is undefined
        let world_cell = world_cell.max(0.001);
        // the sway is a fraction of a cell; past a quarter of one it stops
        // breathing and starts reading as a sliding field again
        let drift = drift.clamp(0.0, 0.25);
        // mean seconds of the birth/death cycle; 0 disables the fade
        let life = life.max(0.0);
        // layer size/alpha mults are relative to the base daub regime; the
        // two split values tile the contrast bands [0, low], [low, high],
        // [high, 1], and the boundaries must never flip
        let mass_mult = mass_mult.clamp(0.05, 8.0);
        let mid_mult = mid_mult.clamp(0.05, 8.0);
        let fine_mult = fine_mult.clamp(0.05, 8.0);
        let mass_alpha = mass_alpha.clamp(0.0, 1.0);
        let mid_alpha = mid_alpha.clamp(0.0, 1.0);
        let fine_alpha = fine_alpha.clamp(0.0, 1.0);
        let contrast_split_low = contrast_split_low.clamp(0.0, 1.0);
        let contrast_split_high = contrast_split_high.clamp(0.0, 1.0).max(contrast_split_low);
        // composite the tiling gates into the layer vec4 (mult, alpha, lo, hi)
        let layers_new = [
            [mass_mult, mass_alpha, 0.0, contrast_split_low],
            [mid_mult, mid_alpha, contrast_split_low, contrast_split_high],
            [fine_mult, fine_alpha, contrast_split_high, 1.0],
        ];

        if self.params_uniform.size == size
            && self.params_uniform.size_min == size_min
            && self.params_uniform.mag_scale == mag_scale
            && self.params_uniform.elongation == elongation
            && self.params_uniform.aspect_min == aspect_min
            && self.params_uniform.alpha == alpha
            && self.params_uniform.feather == feather
            && self.params_uniform.keep == keep
            && self.params_uniform.jitter == jitter
            && self.params_uniform.wobble == wobble
            && self.params_uniform.seed == seed
            && self.params_uniform.flow == flow
            && self.params_uniform.bristle == bristle
            && self.params_uniform.color_jitter == color_jitter
            && self.params_uniform.world_size == world_size
            && self.params_uniform.world_cell == world_cell
            && self.params_uniform.drift == drift
            && self.params_uniform.life == life
            && self.params_uniform.layers == layers_new
        {
            return;
        }

        self.params_uniform = StrokeUniform {
            size,
            size_min,
            mag_scale,
            elongation,
            aspect_min,
            alpha,
            feather,
            keep,
            jitter,
            wobble,
            seed,
            viewport_count: self.params_uniform.viewport_count, // synced separately
            flow,
            bristle,
            color_jitter,
            world_size,
            time: self.params_uniform.time, // rolled separately, every frame
            world_cell,
            drift,
            life,
            layers: layers_new,
            vp: self.params_uniform.vp, // from the graphics mappings
        };

        let s = Services::get();
        let queue = s.gpu.queue();
        queue.write_buffer(&self.params, 0, cast_slice(&[self.params_uniform]));
    }
}

impl RenderFeaturePostProcess for RenderFeaturePostProcessPainterly {
    fn update_params(&mut self, ledger: &mut Vec<Ledger>, mappings: &[GraphicsMapping]) {
        // stamps are clipped to these rects, so they can never paint across the
        // split-screen seam; sync even with no ledger present
        let mut vp_new = self.params_uniform.vp;
        for (i, mapping) in mappings.iter().take(4).enumerate() {
            vp_new[i] = [
                mapping.viewport_min.x,
                mapping.viewport_min.y,
                mapping.viewport_max.x,
                mapping.viewport_max.y,
            ];
        }
        let count_new = mappings.len().clamp(1, 4) as f32;
        if self.params_uniform.viewport_count != count_new
            || self.params_uniform.vp != vp_new
        {
            self.params_uniform.viewport_count = count_new;
            self.params_uniform.vp = vp_new;

            let s = Services::get();
            let queue = s.gpu.queue();
            queue.write_buffer(&self.params, 0, cast_slice(&[self.params_uniform]));
        }

        let Some(ledger) = ledger.first() else { return };

        // world anchor: the SAME camera record the 3D pass bound, read straight
        // from the ledger. the post-process runs on the whole capture, so the
        // projection uses the capture extent; per-seat split screens would
        // want one anchor per seat, but the vp rects keep the seats visually
        // clean today and a shared anchor is the honest single-camera path
        let state_camera = ledger.read::<SysRecordCamera>();
        let s = Services::get();
        let cap_w = s.gpu.capture_width as i32;
        let cap_h = s.gpu.capture_height as i32;
        let cam_uniform = state_camera.cameras.get_uniform(cap_w, cap_h);
        let view: Matrix4<f32> = cam_uniform.view_proj.into();
        // a singular matrix (a degenerate frustum) falls back to identity and
        // degrades to the screen anchor the WGSL far test then produces
        let inv = view.invert().unwrap_or(Matrix4::identity());
        // the projection's y scale survives the GL -> WGPU z rewrite untouched,
        // so [1][1] is 1 / tan(fovy / 2) and the capture focal length follows
        let proj_cols: [[f32; 4]; 4] = state_camera.cameras.get_projection(cap_w, cap_h).calc_matrix().into();
        let anchor_new = WorldAnchorUniform {
            view_proj: cam_uniform.view_proj,
            inv_view_proj: inv.into(),
            cam_pos: cam_uniform.view_position,
            focal_px: 0.5 * cap_h as f32 * proj_cols[1][1],
            pad_x: 0.0,
            pad_y: 0.0,
            pad_z: 0.0,
        };
        if self.world_anchor_uniform != anchor_new {
            self.world_anchor_uniform = anchor_new;
            let queue = s.gpu.queue();
            queue.write_buffer(&self.world_anchor, 0, cast_slice(&[self.world_anchor_uniform]));
        }

        let record = ledger.read::<SysRecordStrokePaint>();

        // the rolling clock only feeds the temporal drift of world-anchored
        // daubs: the WGSL never reads absolute seconds, only a monotone roll,
        // so a fixed 60 Hz step is enough and no clock source is needed
        self.stroke_time += 1.0 / 60.0;
        if self.params_uniform.time != self.stroke_time {
            self.params_uniform.time = self.stroke_time;
            let queue = s.gpu.queue();
            queue.write_buffer(&self.params, 0, cast_slice(&[self.params_uniform]));
        }

        self.update(
            record.size,
            record.size_min,
            record.mag_scale,
            record.elongation,
            record.aspect_min,
            record.alpha,
            record.feather,
            record.keep,
            record.jitter,
            record.wobble,
            record.seed,
            record.flow,
            record.bristle,
            record.color_jitter,
            record.world_size,
            record.world_cell,
            record.drift,
            record.life,
            record.mass_mult,
            record.mass_alpha,
            record.mid_mult,
            record.mid_alpha,
            record.fine_mult,
            record.fine_alpha,
            record.contrast_split_low,
            record.contrast_split_high,
        );
    }

    fn render(&mut self, encoder: &mut wgpu::CommandEncoder, _input_view: &wgpu::TextureView, output_view: &wgpu::TextureView, postprocess_source: PostProcessSource) {
        let bind_group: &BindGroup;
        match postprocess_source {
            PostProcessSource::Offscreen => bind_group = &self.bind_group_input,
            PostProcessSource::ViewA => bind_group = &self.bind_group_a,
            PostProcessSource::ViewB => bind_group = &self.bind_group_b,
        }

        // the guide is laid down first: the daubs are an overlay, not a cover,
        // which is what makes the hybrid read as one painting instead of a
        // scatter of islands on black
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("painterly blit pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(egui_wgpu::wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.blit_pipeline);
            pass.set_bind_group(0, bind_group, &[]);
            pass.draw(0..3, 0..1);
        } // the pass must end here: the stamp pass LOADs what this one stored

        // separate pass because the blend state differs: LOAD keeps the guide
        // that the blit just wrote as the backdrop for the stamps
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("painterly stamp pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output_view,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&self.stamp_pipeline);
        pass.set_bind_group(0, bind_group, &[]);
        pass.draw(0..STAMP_VERTICES, 0..1);
    }
}
