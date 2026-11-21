use camera::SysRecordCamera;
use curio_core::{GraphicsMapping, Ledger, Matrix4x4, Services, TextureAsset};
use lighting::SysRecordSun;

use crate::SysRecordOilPaint;
use egui_wgpu::wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendState, Buffer, BufferAddress, BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites, CommandEncoder, CompareFunction, Device, Extent3d, FragmentState, LoadOp, Operations, PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor, RenderPassDepthStencilAttachment, RenderPipeline, RenderPipelineDescriptor, PipelineLayout, PipelineLayoutDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages, StoreOp, TextureFormat, TextureView, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode};
use egui_wgpu::wgpu::{BufferBinding, BufferBindingType};
use bytemuck::{self};

// ─── dab: one world-pinned brushstroke ────────────────────────────────────
// 48 B; the layout is pinned now so later phases never re-pack the
// instance stream: phase 4 multiplies pin/dir by the model matrix in the
// vertex shader, phase 5 displaces pin by wind, phase 6 re-uploads per
// frame for fire.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Dab {
    pub pin: [f32; 3], // object-LOCAL space
    pub dir: [f32; 3], // which way the stroke runs
    pub size: [f32; 2], // across, along — metres (anisotropy = brush feel)
    pub seed: [f32; 4], // x hue/palette slot, yzw per-dab randoms
}

impl Dab {
    pub fn desc() -> VertexBufferLayout<'static> {
        egui_wgpu::wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Dab>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &[
                // pin, dir, size, seed — offsets mirror the struct above
                egui_wgpu::wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: VertexFormat::Float32x3,
                },
                egui_wgpu::wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 2,
                    format: VertexFormat::Float32x3,
                },
                egui_wgpu::wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 3,
                    format: VertexFormat::Float32x2,
                },
                egui_wgpu::wgpu::VertexAttribute {
                    offset: 36,
                    shader_location: 4,
                    format: VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct QuadCorner {
    pub corner: [f32; 2], // [-0.5, 0.5]; x runs ALONG the stroke
}
impl QuadCorner {
    pub fn desc() -> VertexBufferLayout<'static> {
        egui_wgpu::wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<QuadCorner>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                egui_wgpu::wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x2,
                },
            ],
        }
    }
}

// group 0 — our own per-screen scene uniform. the engine's CameraUniform
// carries view_proj only; strokes also need the camera BASIS so the vertex
// shader can orient them in the view plane without extracting V. sun feeds
// the phase-2 shading term; misc.x is a clock (tick count — no engine time
// yet) and misc.y the screen's pixel height, both reserved for wind (P5)
// until then misc.y just drives pixel-size convergence. 144 B.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct SceneUniform {
    pub view_proj: [[f32; 4]; 4],
    pub view_pos: [f32; 4],
    pub cam_right: [f32; 4],
    pub cam_up: [f32; 4],
    pub sun: [f32; 4], // normalized sun direction
    pub misc: [f32; 4], // x time (ticks), y pixel height
}

pub trait OilPaintFeature {
    fn draw_dabs(&mut self, encoder: &mut CommandEncoder, target_view: &TextureView, ledger: &Ledger, mappings: &[GraphicsMapping], screen: usize);
}

pub struct RenderFeatureDrawDabs {
    pub tick_count: usize,
    pub dabs: Vec<Dab>, // submission list — phase 3 grows this from facets
    pub models: Vec<Matrix4x4>, // parallel to dabs — phase 4 uploads these
    // as the second instance attribute so painted forms pin rigidly on the
    // GPU (no CPU bake)
    pub instance_capacity: usize, // dab_buffer/models_buffer's instance
    // count — grown in powers when authored paint exceeds it

    // built on the first frame that has dabs
    pipeline: Option<RenderPipeline>,
    bind_group: Option<BindGroup>,
    bind_group_layout: Option<BindGroupLayout>, // kept alive by handle
    pipeline_layout: Option<PipelineLayout>, // ditto
    scene_buffer: Option<Buffer>,
    quad_buffer: Option<Buffer>,
    index_buffer: Option<Buffer>,
    dab_buffer: Option<Buffer>,
    models_buffer: Option<Buffer>, // parallel instance stream: one Matrix4x4
    // per dab (phase 4 rigid pin)
    depth: Option<TextureAsset>, // own depth — the mesh pass' texture is
    // local to draw_3d_features and already gone when this pass runs
}

impl RenderFeatureDrawDabs {
    // nothing pre-seeded — the proof patch materializes in draw_dabs only
    // while nothing authored paint, alongside identity pins
    pub fn new() -> RenderFeatureDrawDabs {
        Self::blank()
    }
    pub fn blank() -> RenderFeatureDrawDabs {
        RenderFeatureDrawDabs {
            tick_count: 0,
            dabs: Vec::new(),
            models: Vec::new(),
            instance_capacity: 0,
            pipeline: None,
            bind_group: None,
            bind_group_layout: None,
            pipeline_layout: None,
            scene_buffer: None,
            quad_buffer: None,
            index_buffer: None,
            dab_buffer: None,
            models_buffer: None,
            depth: None,
        }
    }

    // ── phase 1: geometry proof, no sampling yet ────────────────────────
    // ground patch with a wavy flow field + vertical band of tall strokes.
    // done = every dab's angle and width/length ratio visibly differs and
    // the patch welds to the ground as the camera orbits.
    fn proof_dabs() -> Vec<Dab> {
        let mut dabs = Vec::<Dab>::new();
        let mut i = 0;

        for iz in 0..10 {
            for ix in 0..16 {
                let a = Self::hash01(i as u64, 1);
                let b = Self::hash01(i as u64, 2);
                let c = Self::hash01(i as u64, 3);
                let d = Self::hash01(i as u64, 4);

                let fx = ix as f32;
                let fz = iz as f32;
                let flow = 0.7 * ((fz * 0.55 - 2.5) * 0.8).sin() + (c - 0.5) * 1.6;

                dabs.push(
                    Dab {
                        pin: [
                            (fx - 8.0) * 0.55 + (a - 0.5) * 0.3,
                            0.03,
                            (fz - 5.0) * 0.55 + (b - 0.5) * 0.3,
                        ],
                        dir: [flow.cos(), 0.0, flow.sin()],
                        size: [0.26 + d * 0.2, 0.5 + a * 0.35],
                        seed: [Self::hash01(i as u64, 5), a, b, c],
                    },
                );
                i += 1;
            }
        }

        for iy in 0..6 {
            for ix in 0..14 {
                let a = Self::hash01(i as u64, 1);
                let b = Self::hash01(i as u64, 2);
                let c = Self::hash01(i as u64, 3);
                let d = Self::hash01(i as u64, 4);

                let flow = 1.5708 + (c - 0.5) * 1.1;

                let fx = ix as f32;
                let fy = iy as f32;

                dabs.push(
                    Dab {
                        pin: [
                            (fx - 7.0) * 0.45 + (a - 0.5) * 0.2,
                            0.35 + fy * 0.5 + (b - 0.5) * 0.2,
                            -5.5,
                        ],
                        dir: [flow.cos(), flow.sin(), 0.0],
                        size: [0.3 + d * 0.16, 0.62 + a * 0.4],
                        seed: [Self::hash01(i as u64, 6), a, b, c],
                    },
                );
                i += 1;
            }
        }

        dabs
    }
    pub fn hash01(i: u64, salt: u64) -> f32 {
        // splitmix finalizer — wrapping so debug builds don't trap
        let mut x = i.wrapping_mul(0x9E3779B97F4A7C15) + salt;
        x ^= x >> 30;
        x = x.wrapping_mul(0xBF58476D1CE4E5B9);
        x ^= x >> 27;
        x = x.wrapping_mul(0x94D049BB133111EB);
        x ^= x >> 31;
        return (x >> 40) as f32 / 0xFFFFFF as f32;
    }

    // ── phase 4: rigid pin, no CPU bake ─────────────────────────────────
    // the record's dabs (object-local) and their parallel form matrices go
    // straight to the GPU as two instance streams; the vertex shader
    // multiplies pin with w=1 and dir with w=0, and the stroke picks up
    // the matrix's mean basis-column length as scale. A painted form
    // translating/scaling therefore costs only its Transform3D update —
    // the paint never re-samples

    // vertex buffers cannot be resized in place — re-create both at the
    // new capacity. Nothing to preserve: draw_dabs re-uploads right after
    pub fn grow_instance_buffers(&mut self, device: &Device) {
        let want = self.dabs.len();
        let mut capacity = if self.instance_capacity < 1 { 1 } else { self.instance_capacity };
        while capacity < want {
            capacity = capacity * 2;
        }
        self.instance_capacity = capacity;
        self.dab_buffer = Some(
            device.create_buffer(&BufferDescriptor {
                label: Some("dab instances"),
                size: (capacity * std::mem::size_of::<Dab>()) as u64,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        );
        self.models_buffer = Some(
            device.create_buffer(&BufferDescriptor {
                label: Some("dab model matrices"),
                size: (capacity * std::mem::size_of::<Matrix4x4>()) as u64,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        );
    }

    // ── frame pass ──────────────────────────────────────────────────────
    // pub so the rendering crate can call it directly; the trait below
    // delegates, mirroring the ui_overlay precedent
    pub fn draw_dabs(&mut self, encoder: &mut CommandEncoder, target_view: &TextureView, ledger: &Ledger, mappings: &[GraphicsMapping], screen: usize) {
        self.tick_count += 1;

        // Paint source: this screen's OilPaint record — the habit refills
        // it each tick from the world's PaintOil forms. Dabs and their
        // form matrices go straight to the GPU as two parallel instance
        // streams (phase 4 rigid pin: moving a painted form moves its
        // strokes for free, no CPU bake, no re-sampling). Until anything
        // authored exists, the phase-1 proof patch stays on the
        // submission list with identity pins.
        let paint = ledger.read::<SysRecordOilPaint>();
        if !paint.dabs.is_empty() {
            self.dabs = paint.dabs.to_vec();
            self.models = paint.models.to_vec();
        } else if self.dabs.is_empty() {
            self.dabs = Self::proof_dabs();
            let id = Matrix4x4::from_raw(
                [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
            );
            while self.models.len() < self.dabs.len() {
                self.models.push(id);
            }
        }

        if self.dabs.is_empty() || screen >= mappings.len() {
            return;
        }

        let s = Services::get();
        let device = s.gpu.device();
        let queue = s.gpu.queue();

        if self.pipeline.is_none() {
            self.init_gpu(device);
        }

        // instance streams: padded capacity, grown in powers when the
        // authored paint outgrows it, re-uploaded every frame (the record
        // is refilled each tick)
        if self.dabs.len() > self.instance_capacity {
            self.grow_instance_buffers(device);
        }
        if let buffer = self.dab_buffer.as_ref().unwrap() {
            if (self.dabs.len() * std::mem::size_of::<Dab>()) as u64 > buffer.size() {
                println!("[gpu-diag] dab instances size={} write={}", buffer.size(), self.dabs.len() * std::mem::size_of::<Dab>());
            }
        }
        if let buffer = self.models_buffer.as_ref().unwrap() {
            if (self.models.len() * std::mem::size_of::<Matrix4x4>()) as u64 > buffer.size() {
                println!(
                    "[gpu-diag] dab models size={} write={} dabs={} models={}",
                    buffer.size(),
                    self.models.len() * std::mem::size_of::<Matrix4x4>(),
                    self.dabs.len(),
                    self.models.len(),
                );
            }
        }
        queue.write_buffer(self.dab_buffer.as_ref().unwrap(), 0, bytemuck::cast_slice(&self.dabs));
        queue.write_buffer(self.models_buffer.as_ref().unwrap(), 0, bytemuck::cast_slice(&self.models));

        // first frame: tell the user where the sample curio camera sits so
        // the proof patch can be judged on their machine
        if self.tick_count == 1 {
            let state_camera = ledger.read::<SysRecordCamera>();
            println!(
                "[oil-paint] {} dabs armed; camera at ({:.2},{:.2},{:.2})",
                self.dabs.len(),
                state_camera.cameras.position.x,
                state_camera.cameras.position.y,
                state_camera.cameras.position.z,
            );
        }

        // scene uniform for this screen: engine get_uniform for the
        // matrices, view-basis rows for stroke orientation
        let state_camera = ledger.read::<SysRecordCamera>();
        let snapshot = &state_camera.cameras;

        let width = s.gpu.capture_width as f32;
        let height = s.gpu.capture_height as f32;
        let mapping = &mappings[screen];
        let vx = mapping.viewport_min.x * width;
        let vy = mapping.viewport_min.y * height;
        let vw = ((mapping.viewport_max.x - mapping.viewport_min.x) * width).round();
        let vh = ((mapping.viewport_max.y - mapping.viewport_min.y) * height).round();

        let cam = snapshot.get_uniform(vw as i32, vh as i32);
        let view = snapshot.calc_matrix(); // cgmath columns: row i is [col][i]

        // sun for the dab shading term — normalized so the fs term is a
        // pure flow-against-light blend
        let state_sun = ledger.read::<SysRecordSun>();
        let sd = state_sun.direction;
        let sun_len = (sd.x * sd.x + sd.y * sd.y + sd.z * sd.z).sqrt();
        let sun_inv = if sun_len < 0.0001 { 0.0 } else { 1.0 / sun_len };

        let scene = SceneUniform {
            view_proj: cam.view_proj,
            view_pos: cam.view_position,
            cam_right: [view[0][0], view[1][0], view[2][0], 0.0],
            cam_up: [view[0][1], view[1][1], view[2][1], 0.0],
            sun: [sd.x * sun_inv, sd.y * sun_inv, sd.z * sun_inv, 0.0],
            misc: [self.tick_count as f32, vh, 0.0, 0.0],
        };
        if std::mem::size_of::<SceneUniform>() as u64 > self.scene_buffer.as_ref().unwrap().size() {
            println!("[gpu-diag] dab scene size={} write={}", self.scene_buffer.as_ref().unwrap().size(), std::mem::size_of::<SceneUniform>());
        }
        queue.write_buffer(self.scene_buffer.as_ref().unwrap(), 0, bytemuck::cast_slice(&[scene]));

        // second 3D pass on the same offscreen: LOAD keeps what the mesh
        // pass painted; own depth starts clean (see lib.rs note about the
        // mesh depth texture being local to draw_3d_features)
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("oil paint dab pass"),
            color_attachments: &[
                Some(RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    ops: Operations { load: LoadOp::Load, store: StoreOp::Store },
                }),
            ],
            depth_stencil_attachment: Self::depth_attachment(self.depth.as_ref().unwrap()),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(self.pipeline.as_ref().unwrap());
        pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);
        pass.set_viewport(vx, vy, vw, vh, 0.0, 1.0);
        pass.set_vertex_buffer(0, self.quad_buffer.as_ref().unwrap().slice(..));
        pass.set_vertex_buffer(1, self.dab_buffer.as_ref().unwrap().slice(0..(self.dabs.len() as BufferAddress * std::mem::size_of::<Dab>() as BufferAddress)));
        // phase 4 rigid pin: one model matrix per dab, sliced to the very
        // same instance count — the two streams grow and upload together
        pass.set_vertex_buffer(2, self.models_buffer.as_ref().unwrap().slice(0..(self.dabs.len() as BufferAddress * std::mem::size_of::<Matrix4x4>() as BufferAddress)));
        pass.set_index_buffer(self.index_buffer.as_ref().unwrap().slice(..), egui_wgpu::wgpu::IndexFormat::Uint32);

        pass.draw_indexed(0..6, 0, 0..(self.dabs.len() as u32));
    }
    pub fn depth_attachment<'a>(depth: &'a TextureAsset) -> Option<RenderPassDepthStencilAttachment<'a>> {
        Some(
            RenderPassDepthStencilAttachment {
                view: &depth.view,
                depth_ops: Some(Operations { load: LoadOp::Clear(1.0), store: StoreOp::Store }),
                stencil_ops: None,
            },
        )
    }

    fn init_gpu(&mut self, device: &Device) {
        let queue = Services::get().gpu.queue();
        // two triangles per dab quad; corner is [-0.5,0.5], x along stroke.
        // written once — the instance streams grow in powers via
        // grow_instance_buffers instead
        let quad = [
            [-0.5, -0.5],
            [0.5, -0.5],
            [0.5, 0.5],
            [-0.5, 0.5],
        ];
        let indices = [0, 1, 2, 0, 2, 3];

        self.quad_buffer = Some(
            device.create_buffer(&BufferDescriptor {
                label: Some("dab quad corners"),
                size: (quad.len() * std::mem::size_of::<QuadCorner>()) as u64,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        );
        if let buffer = self.quad_buffer.as_ref().unwrap() {
            if (quad.len() * std::mem::size_of::<QuadCorner>()) as u64 > buffer.size() {
                println!("[gpu-diag] dab quad corners size={} write={}", buffer.size(), quad.len() * std::mem::size_of::<QuadCorner>());
            }
        }
        queue.write_buffer(self.quad_buffer.as_ref().unwrap(), 0, bytemuck::cast_slice(&quad));

        self.index_buffer = Some(
            device.create_buffer(&BufferDescriptor {
                label: Some("dab indices"),
                size: (indices.len() * std::mem::size_of::<u32>()) as u64,
                usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        );
        if (indices.len() * std::mem::size_of::<u32>()) as u64 > self.index_buffer.as_ref().unwrap().size() {
            println!("[gpu-diag] dab indices size={} write={}", self.index_buffer.as_ref().unwrap().size(), indices.len() * std::mem::size_of::<u32>());
        }
        queue.write_buffer(self.index_buffer.as_ref().unwrap(), 0, bytemuck::cast_slice(&indices));

        // instance streams share one padded capacity so they can never
        // desync; grow_instance_buffers doubles both together. No upload
        // here — draw_dabs uploads both after the growth check, every frame
        self.instance_capacity = if self.dabs.len() < 4096 { 4096 } else { self.dabs.len() };
        self.dab_buffer = Some(
            device.create_buffer(&BufferDescriptor {
                label: Some("dab instances"),
                size: (self.instance_capacity * std::mem::size_of::<Dab>()) as u64,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        );
        self.models_buffer = Some(
            device.create_buffer(&BufferDescriptor {
                label: Some("dab model matrices"),
                size: (self.instance_capacity * std::mem::size_of::<Matrix4x4>()) as u64,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        );

        self.scene_buffer = Some(
            device.create_buffer(&BufferDescriptor {
                label: Some("dab scene uniform"),
                size: std::mem::size_of::<SceneUniform>() as u64,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        );

        // 1280×720 matches the fixed offscreen the rendering crate makes
        let depth_texture = device.create_texture(&egui_wgpu::wgpu::TextureDescriptor {
            label: Some("dab depth"),
            size: Extent3d { width: 1280, height: 720, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: egui_wgpu::wgpu::TextureDimension::D2,
            format: TextureAsset::DEPTH_FORMAT,
            usage: egui_wgpu::wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&egui_wgpu::wgpu::TextureViewDescriptor::default());
        // sampler unused (attachment only) but the TextureAsset shape keeps
        // one, like the 3D helper's own depth texture
        let depth_sampler = device.create_sampler(&egui_wgpu::wgpu::SamplerDescriptor::default());
        self.depth = Some(
            TextureAsset {
                texture: depth_texture,
                view: depth_view,
                sampler: depth_sampler,
            },
        );

        // pipeline: one shader module, quad + instance buffers, own depth.
        // cull off — dabs are single-layer and view-facing by construction
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("dab shader"),
            source: ShaderSource::Wgsl(include_str!("dab.wgsl").into()),
        });

        self.bind_group_layout = Some(
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("dab scene layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<SceneUniform>() as u64),
                        },
                        count: None,
                    },
                ],
            }),
        );
        self.bind_group = Some(
            device.create_bind_group(&BindGroupDescriptor {
                label: Some("dab scene"),
                layout: &self.bind_group_layout.as_ref().unwrap(),
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::Buffer(BufferBinding {
                            buffer: self.scene_buffer.as_ref().unwrap(),
                            offset: 0,
                            size: None,
                        }),
                    },
                ],
            }),
        );

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("dab pipeline layout"),
            bind_group_layouts: &[&self.bind_group_layout.as_ref().unwrap()],
            push_constant_ranges: &[],
        });

        let mut primitive: PrimitiveState = PrimitiveState::default();
        primitive.cull_mode = None; // strokes double-side themselves

        self.pipeline = Some(
            device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("dab pipeline"),
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[QuadCorner::desc(), Dab::desc(), Matrix4x4::desc()],
                    compilation_options: Default::default(),
                },
                fragment: Some(
                    FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"),
                        targets: &[
                            Some(ColorTargetState {
                                format: TextureFormat::Rgba8UnormSrgb,
                                blend: Some(BlendState::REPLACE), // near-opaque paint: no sorting
                                write_mask: ColorWrites::ALL,
                            }),
                        ],
                        compilation_options: Default::default(),
                    },
                ),
                primitive,
                depth_stencil: Some(
                    egui_wgpu::wgpu::DepthStencilState {
                        format: TextureAsset::DEPTH_FORMAT,
                        depth_write_enabled: true,
                        depth_compare: CompareFunction::Less,
                        stencil: egui_wgpu::wgpu::StencilState::default(),
                        bias: egui_wgpu::wgpu::DepthBiasState::default(),
                    },
                ),
                multisample: egui_wgpu::wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            }),
        );
        // the pipeline and bind group reference layouts by handle, so keep
        // them alive (ui_overlay does the same)
        self.pipeline_layout = Some(pipeline_layout);
    }
}

impl OilPaintFeature for RenderFeatureDrawDabs {
    fn draw_dabs(&mut self, encoder: &mut CommandEncoder, target_view: &TextureView, ledger: &Ledger, mappings: &[GraphicsMapping], screen: usize) {
        // concrete impl on the type wins — no recursion
        self.draw_dabs(encoder, target_view, ledger, mappings, screen);
    }
}

