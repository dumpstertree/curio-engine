use curio_core::{GraphicsMapping, Ledger, Services};
use egui_wgpu::wgpu::{self, BindGroup};
use egui_wgpu::wgpu::util::DeviceExt;
use std::borrow::Cow;
use std::num::NonZeroU64;
use std::sync::Arc;

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
const STAMP_SLOTS: u32 = 1024; // must match SLOT_COUNT in the WGSL
const STAMP_VERTICES: u32 = 6 * STAMP_SLOTS; // two triangles per slot

/// CPU-side mirror of the WGSL StrokeParams struct: 12 scalars then 4 pads so
/// the vp composite lands on offset 64 exactly as naga lays it out (128 bytes)
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct StrokeUniform {
    size: f32, // long-axis daub length in flat areas, px
    size_min: f32, // fraction of size kept where local contrast is maxed
    mag_scale: f32, // guide-contrast sensitivity: gradient * this -> 0..1
    elongation: f32, // long/short aspect in flat areas
    aspect_min: f32, // aspect floor in busy areas
    alpha: f32, // stamp opacity
    feather: f32, // 0 = hard-edged stamp, 1 = gradient all the way in
    keep: f32, // keep probability per slot; failures degenerate
    jitter: f32, // anchor jitter as a fraction of one slot cell
    wobble: f32, // angle jitter amplitude in radians
    seed: f32, // shifts every hash
    viewport_count: f32, // number of valid vp rects (1..4)
    pad_x: f32,
    pad_y: f32,
    pad_z: f32,
    pad_w: f32,
    vp: [[f32; 4]; 4], // viewport rects in uv: minx, miny, maxx, maxy
}

pub struct RenderFeaturePostProcessPainterly {
    blit_pipeline: wgpu::RenderPipeline, // guide -> output, REPLACE
    stamp_pipeline: wgpu::RenderPipeline, // instanced daubs, ALPHA_BLENDING
    bind_group_input: wgpu::BindGroup, // guide = the original offscreen_view
    bind_group_a: wgpu::BindGroup, // guide = view_a
    bind_group_b: wgpu::BindGroup, // guide = view_b
    params: wgpu::Buffer, // StrokeUniform, rewritten per frame
    params_uniform: StrokeUniform,
}

impl RenderFeaturePostProcessPainterly {
    pub fn new(
        device: Arc<wgpu::Device>,
        format: wgpu::TextureFormat,
        post_resources: &PostProcessResources,
        // the stamp pass sobels the GUIDE, never the depth buffer: keeping the
        // pass free of kuwahara's flow textures is what lets the two features
        // own their resources independently
        _depth_view: &wgpu::TextureView,
        offscreen_view: &wgpu::TextureView,
    ) -> Box<Self> {
        let params_uniform = StrokeUniform {
            // reference-scale daubs: long axis 22 px in flat areas, 7-8 px in
            // the busiest ones, held along edges at 2.2:1
            size: 22.0,
            size_min: 0.35,
            mag_scale: 14.0, // window-mean gradients damp a touch below 1-tap sobels, so sensitivity is a little higher than the old 10

            elongation: 2.2,
            aspect_min: 1.4,
            alpha: 1.0,
            feather: 0.45,
            keep: 1.0, // every slot draws: coverage is the first thing to see
            jitter: 0.45,
            wobble: 0.25,
            seed: 0.0,
            viewport_count: 1.0, // whole-texture rect until the helper syncs
            pad_x: 0.0,
            pad_y: 0.0,
            pad_z: 0.0,
            pad_w: 0.0,
            vp: [[0.0, 0.0, 1.0, 1.0], [0.0, 0.0, 1.0, 1.0], [0.0, 0.0, 1.0, 1.0], [0.0, 0.0, 1.0, 1.0]],
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
                        min_binding_size: NonZeroU64::new(128),
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
            pad_x: 0.0,
            pad_y: 0.0,
            pad_z: 0.0,
            pad_w: 0.0,
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
        let record = ledger.read::<SysRecordStrokePaint>();
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
