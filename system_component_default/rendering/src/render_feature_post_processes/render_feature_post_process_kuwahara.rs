use curio_core::{GraphicsMapping, Ledger, Services};
use egui_wgpu::wgpu::{self, BindGroup};
use egui_wgpu::wgpu::util::DeviceExt;
use std::borrow::Cow;
use std::num::NonZeroU64;
use std::sync::Arc;

use bytemuck::{Pod, Zeroable, cast_slice};

use crate::PostProcessResources;
use crate::record::sys_record_post_process::SysRecordPostProcess;
use crate::render_feature_post_process::{PostProcessSource, RenderFeaturePostProcess};

// NOTE: update include_str! path to your shader file
// const FOG_WGSL_PATH: &str = "../postprocess_fog.wgsl";

const DITHER_COMMON_WGSL: &str = include_str!("../dither_common.wgsl");
const KUWAHARA_WGSL: &str = include_str!("../postprocess_kuwahara.wgsl");
const FLOW_FIELD_WGSL: &str = include_str!("../flow_field.wgsl");

/// CPU-side mirror of the WGSL KuwaharaParams struct
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct KuwaharaUniform {
    radius: f32, // kernel radius in pixels
    strength: f32, // mix between the original color and the painterly color
    sigma: f32, // gaussian spread ACROSS the detected edge (px); 0 disables the weighting
    tau: f32, // gaussian spread ALONG the detected edge (px); 0 disables the weighting
    dither_mode: f32, // 0 = interleaved gradient noise, >= 1 = hashed blue noise
    dither_amount: f32, // 0 = hard decisions (no dithering), 1 = fully stochastic
    seed: f32, // shifts the hash lattice (also usable for temporal dithering)
    depth_strength: f32, // 0 = depth-ignorant; > 0 window weights fall off across depth jumps
    temporal: f32, // 0 = static spatial dither; > 0 re-randomizes the noise each frame
    frame: f32, // CPU frame counter driving the re-randomization (wraps at 1024)
    stroke: f32, // stretch sub-windows ALONG the edge (0 = square, 2 = 3x long)
    stride: f32, // sampling step inside windows; 0 = auto (grows with window)
    detail: f32, // flow detail-shrink sensitivity; 0 = OFF, strokes keep full
        // lattice size everywhere (what the references do)
    viewport_count: f32, // number of valid vp rects (1..4)
    pad_x: f32, // scalar pads: uniform-space composites align to 16, so vp lands
    pad_y: f32, // at offset 64 on the GPU; the #[repr(C)] mirror pads identically
    vp: [[f32; 4]; 4], // viewport rects in uv: minx, miny, maxx, maxy
}

// CPU-side mirror of the WGSL FlowParams struct (16 bytes)
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct FlowUniform {
    dx: f32, // blur axis x component
    dy: f32, // blur axis y component
    k_depth: f32, // weight of the depth-gradient tensor in the flow field
    pad: f32,
}

pub struct RenderFeaturePostProcessKuwahara {
    pipeline: wgpu::RenderPipeline,
    // half-res guide chain: the kernel and the flow field run on a half-
    // capture grid (~quartered pixel cost); a linear blit brings the chain's
    // current source DOWN onto the guide grid, and another blit hands the
    // finished guide back UP to the full-res post target so the chain
    // contract downstream is untouched. the guide carries only flat patch
    // colours, crispness lives in the full-res stamp/outline layers, so
    // bilinear both ways reads artefact-free
    blit_pipeline: wgpu::RenderPipeline, // linear copy, down AND up
    bind_group_kernel: wgpu::BindGroup, // kernel always reads guide_view_src
    blit_bg_input: wgpu::BindGroup, // offscreen -> guide grid (downsample)
    blit_bg_a: wgpu::BindGroup, // view_a -> guide grid
    blit_bg_b: wgpu::BindGroup, // view_b -> guide grid
    blit_bg_guide: wgpu::BindGroup, // guide grid -> chain target (upsample)
    guide_texture_src: wgpu::Texture, // half-res downsampled chain source
    guide_view_src: wgpu::TextureView,
    guide_texture_dst: wgpu::Texture, // half-res kuwahara output
    guide_view_dst: wgpu::TextureView,
    params: wgpu::Buffer, // KuwaharaUniform, rewritten per frame
    params_uniform: KuwaharaUniform,
    frame_index: f32, // internal temporal-dither counter (wraps at 1024)
    // flow field: sobel gradients + separable blur, computed once per frame
    // from the DOWNSAMPLED frame on the half grid; the kernel reads it with
    // one textureLoad
    flow_pipeline: wgpu::RenderPipeline, // sobel pass
    blur_pipeline: wgpu::RenderPipeline, // separable blur, run h then v
    flow_texture_src: wgpu::Texture, // (E,F,G) tensor, Rgba16Float, half res
    flow_texture_tmp: wgpu::Texture, // intermediate for the second blur leg
    flow_view_src: wgpu::TextureView,
    flow_view_tmp: wgpu::TextureView,
    flow_params_h: wgpu::Buffer, // dir (1,0), also bound to the sobel pass
    flow_params_v: wgpu::Buffer, // dir (0,1)
    flow_bg_sobel: wgpu::BindGroup, // original color + depth -> flow_texture_src
    flow_bg_h: wgpu::BindGroup, // flow_texture_src -> flow_texture_tmp
    flow_bg_v: wgpu::BindGroup, // flow_texture_tmp -> flow_texture_src
}

impl RenderFeaturePostProcessKuwahara {
    pub fn new(
        device: Arc<wgpu::Device>,
        format: wgpu::TextureFormat,
        post_resources: &PostProcessResources,
        depth_view: &wgpu::TextureView,
        // add offscreen_view here so we can create the initial bind group
        offscreen_view: &wgpu::TextureView,
    ) -> Box<Self> {
        // default parameters - the naive filter's radius of 4, fully painterly
        // (values are already in HALF-RES guide units: the ledger's screen-px
        // defaults radius 4 / sigma 1.5 / tau 3 halved by update())
        let params_uniform = KuwaharaUniform {
            radius: 2.0,
            strength: 1.0,
            sigma: 0.75,
            tau: 1.5,
            dither_mode: 0.0,
            dither_amount: 0.0,
            seed: 0.0,
            depth_strength: 40.0,
            temporal: 0.0,
            frame: 0.0,
            stroke: 0.0, // square windows until the game asks for strokes
            stride: 0.0, // auto stride
            detail: 0.0, // detail-shrink off: strokes keep the full lattice size
            viewport_count: 1.0, // whole-texture rect until the helper syncs
            pad_x: 0.0,
            pad_y: 0.0,
            vp: [[0.0, 0.0, 1.0, 1.0], [0.0, 0.0, 1.0, 1.0], [0.0, 0.0, 1.0, 1.0], [0.0, 0.0, 1.0, 1.0]],
        };

        // sampler
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("kuwahara sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // shader: shared dither helpers prepended to the pass source, so both
        // compile as one module (the dither helpers are reused by other passes)
        let shader_source = format!("{}\n{}", DITHER_COMMON_WGSL, KUWAHARA_WGSL);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("kuwahara shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader_source.as_str())),
        });

        // params uniform buffer
        let params = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("kuwahara params"),
            contents: cast_slice(&[params_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // bind group layout (sampler, color texture, depth texture, params)
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("kuwahara bind group layout"),
            entries: &[
                // sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // color texture (float)
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // depth texture (depth)
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // kuwahara parameters
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(128),
                    },
                    count: None,
                },
                // smoothed flow-field tensor (E,F,G) written by the flow passes
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        // pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("kuwahara pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("kuwahara pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_fullscreen"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_fullscreen"),
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

        // ── half-res guide grid + flow field resources ────────────────
        // sobel gradient tensor (luminance + k * depth) of the DOWNSAMPLED
        // frame, separable-blurred h then v; all of this runs on the half
        // grid: quarter of the pixels, same on-screen behaviour (the CPU
        // halves the ledger's screen-px kernel units to match)
        let s = Services::get();
        let guide_width = (s.gpu.capture_width + 1) / 2;
        let guide_height = (s.gpu.capture_height + 1) / 2;
        let flow_size = wgpu::Extent3d {
            width: guide_width,
            height: guide_height,
            depth_or_array_layers: 1,
        };
        let flow_format = wgpu::TextureFormat::Rgba16Float;
        let flow_usage = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;

        let flow_texture_src = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("flow tensor"),
            size: flow_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: flow_format,
            usage: flow_usage,
            view_formats: &[],
        });
        let flow_view_src = flow_texture_src.create_view(&wgpu::TextureViewDescriptor::default());

        let flow_texture_tmp = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("flow tensor tmp"),
            size: flow_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: flow_format,
            usage: flow_usage,
            view_formats: &[],
        });
        let flow_view_tmp = flow_texture_tmp.create_view(&wgpu::TextureViewDescriptor::default());

        // ── half-res guide pair ───────────────────────────────────────
        // src: the chain source blitted down; dst: the kuwahara kernel's
        // output, blitted back up to the full-res chain target. same sRGB
        // format as the chain so sampling decodes and writing re-encodes
        // exactly like the full-res passes do
        let guide_size = flow_size;
        let guide_usage = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        let guide_desc = |label| wgpu::TextureDescriptor {
            label: Some(label),
            size: guide_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: guide_usage,
            view_formats: &[],
        };

        let guide_texture_src = device.create_texture(&guide_desc("guide half src"));
        let guide_view_src = guide_texture_src.create_view(&wgpu::TextureViewDescriptor::default());

        let guide_texture_dst = device.create_texture(&guide_desc("guide half dst"));
        let guide_view_dst = guide_texture_dst.create_view(&wgpu::TextureViewDescriptor::default());

        let flow_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("flow field shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(FLOW_FIELD_WGSL)),
        });

        // blur axes are static: two small buffers, depth tensor weight k is a
        // constant here (raise it to make strokes follow geometry harder)
        let flow_params_h = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("flow params h"),
            contents: cast_slice(&[FlowUniform { dx: 1.0, dy: 0.0, k_depth: 2.0, pad: 0.0 }]),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let flow_params_v = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("flow params v"),
            contents: cast_slice(&[FlowUniform { dx: 0.0, dy: 1.0, k_depth: 2.0, pad: 0.0 }]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let flow_bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("flow field bind group layout"),
            entries: &[
                // source: original color (sobel) or blurred tensor (blur)
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // depth (sobel only; blur binds it for layout parity)
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // flow parameters
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(16),
                    },
                    count: None,
                },
            ],
        });

        let flow_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("flow field pipeline layout"),
            bind_group_layouts: &[&flow_bind_layout],
            push_constant_ranges: &[],
        });

        let flow_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("flow sobel pipeline"),
            layout: Some(&flow_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &flow_shader,
                entry_point: Some("vs_flow"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &flow_shader,
                entry_point: Some("fs_sobel"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: flow_format,
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

        let blur_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("flow blur pipeline"),
            layout: Some(&flow_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &flow_shader,
                entry_point: Some("vs_flow"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &flow_shader,
                entry_point: Some("fs_blur"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: flow_format,
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

        // ── guide blit: one linear copy pipeline, used down (chain source
        // -> half grid) and up (half grid -> full-res chain target) ─────
        let blit_bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("kuwahara blit bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let blit_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("kuwahara blit pipeline layout"),
            bind_group_layouts: &[&blit_bind_layout],
            push_constant_ranges: &[],
        });

        let blit_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("kuwahara blit pipeline"),
            layout: Some(&blit_pipeline_layout),
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

        let blit_bg_input = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("kuwahara blit bind group INPUT"),
            layout: &blit_bind_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(offscreen_view),
                },
            ],
        });

        let blit_bg_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("kuwahara blit bind group A"),
            layout: &blit_bind_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&post_resources.view_a),
                },
            ],
        });

        let blit_bg_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("kuwahara blit bind group B"),
            layout: &blit_bind_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&post_resources.view_b),
                },
            ],
        });

        let blit_bg_guide = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("kuwahara blit bind group GUIDE"),
            layout: &blit_bind_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&guide_view_dst),
                },
            ],
        });

        let flow_bg_sobel = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("flow sobel bind group"),
            layout: &flow_bind_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    // the DOWNSAMPLED frame, not the full-res scene: the sobel
                    // taps then span 2 capture px and match the guide grid the
                    // kernel reads
                    resource: wgpu::BindingResource::TextureView(&guide_view_src),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &flow_params_h,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        let flow_bg_h = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("flow blur h bind group"),
            layout: &flow_bind_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&flow_view_src),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &flow_params_h,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        let flow_bg_v = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("flow blur v bind group"),
            layout: &flow_bind_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&flow_view_tmp),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &flow_params_v,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        // the kernel always samples the half-res guide grid: the downsample
        // blit already copied the chain's current source there, so the source
        // enum only drives the blit bind group now
        let bind_group_kernel = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("kuwahara bind group KERNEL"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&guide_view_src),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &params,
                        offset: 0,
                        size: None,
                    }),
                },
                // smoothed flow-field tensor (E,F,G)
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&flow_view_src),
                },
            ],
        });

        // (the full-res source bind groups are gone: the kernel reads the
        // guide grid, and the three blit_bg_* groups carry the source choice)

        // store raw pointer ids for quick comparison later
        // let bind_group_input_tex_id = (offscreen_view as *const _) as usize;
        // let bind_group_a_tex_id = (&post_resources.view_a as *const _) as usize;
        // let bind_group_b_tex_id = (&post_resources.view_b as *const _) as usize;

        Box::new(Self {
            pipeline,
            blit_pipeline,
            bind_group_kernel,
            blit_bg_input,
            blit_bg_a,
            blit_bg_b,
            blit_bg_guide,
            guide_texture_src,
            guide_view_src,
            guide_texture_dst,
            guide_view_dst,
            params,
            params_uniform,
            frame_index: 0.0,
            flow_pipeline,
            blur_pipeline,
            flow_texture_src,
            flow_texture_tmp,
            flow_view_src,
            flow_view_tmp,
            flow_params_h,
            flow_params_v,
            flow_bg_sobel,
            flow_bg_h,
            flow_bg_v,
            // bind_group_input_tex_id,
            // bind_group_a_tex_id,
            // bind_group_b_tex_id,
        })
    }

    // rewrite the params uniform, only when something actually changed
    pub fn update(
        &mut self,
        radius: f32,
        strength: f32,
        sigma: f32,
        tau: f32,
        dither_mode: f32,
        dither_amount: f32,
        seed: f32,
        depth_strength: f32,
        temporal: f32,
        frame: f32,
        stroke: f32,
        stride: f32,
        detail: f32,
    ) {
        let radius = radius.max(1.0);
        let strength = strength.clamp(0.0, 1.0);
        let sigma = sigma.max(0.0);
        let tau = tau.max(0.0);
        let dither_mode = dither_mode.max(0.0);
        let dither_amount = dither_amount.clamp(0.0, 1.0);
        let seed = seed % 1024.0; // keep the hash lattice small
        let depth_strength = depth_strength.max(0.0);
        let temporal = temporal.clamp(0.0, 1.0);
        let frame = frame % 1024.0;
        // headroom for reference-scale daubs: stroke 4 = 5x-long windows
        let stroke = stroke.clamp(0.0, 4.0);
        let stride = stride.clamp(0.0, 8.0);
        let detail = detail.max(0.0);
        // the kernel runs on the HALF-RES guide grid: the ledger speaks in
        // SCREEN px, so the window units are halved here to keep tuning
        // numbers describing the on-screen picture (kernel radius floor is
        // one guide texel = 2 capture px)
        let radius_half = (radius * 0.5).max(1.0);
        let sigma_half = sigma * 0.5;
        let tau_half = tau * 0.5;

        if self.params_uniform.radius == radius_half
            && self.params_uniform.strength == strength
            && self.params_uniform.sigma == sigma_half
            && self.params_uniform.tau == tau_half
            && self.params_uniform.dither_mode == dither_mode
            && self.params_uniform.dither_amount == dither_amount
            && self.params_uniform.seed == seed
            && self.params_uniform.depth_strength == depth_strength
            && self.params_uniform.temporal == temporal
            && self.params_uniform.frame == frame
            && self.params_uniform.stroke == stroke
            && self.params_uniform.stride == stride
            && self.params_uniform.detail == detail
        {
            return;
        }

        self.params_uniform = KuwaharaUniform {
            radius: radius_half,
            strength,
            sigma: sigma_half,
            tau: tau_half,
            dither_mode,
            dither_amount,
            seed,
            depth_strength,
            temporal,
            frame,
            stroke,
            stride,
            detail,
            viewport_count: self.params_uniform.viewport_count, // synced separately
            pad_x: 0.0,
            pad_y: 0.0,
            vp: self.params_uniform.vp, // from the graphics mappings
        };

        let s = Services::get();
        let queue = s.gpu.queue();
        queue.write_buffer(&self.params, 0, cast_slice(&[self.params_uniform]));
    }
}

impl RenderFeaturePostProcess for RenderFeaturePostProcessKuwahara {
    // per-frame pickup of the game's tuning record: the single post chain is
    // driven by the first ledger instance; update() clamps and only rewrites
    // the uniform when a value actually changed
    fn update_params(&mut self, ledger: &mut Vec<Ledger>, mappings: &[GraphicsMapping]) {
        // viewport sync first: these rects matter even with no ledger present.
        // kernel taps clamp to them so sub-windows never read colour across
        // the split-screen seam; up to four screens supported
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
        let record = ledger.read::<SysRecordPostProcess>();
        self.update(
            record.radius,
            record.strength,
            record.sigma,
            record.tau,
            record.dither_mode,
            record.dither_amount,
            record.seed,
            record.depth_strength,
            record.temporal,
            self.frame_index, // temporal clock stays internal to the feature
            record.stroke,
            record.stride,
            record.detail,
        );
    }

    fn render(&mut self, encoder: &mut wgpu::CommandEncoder, _input_view: &wgpu::TextureView, output_view: &wgpu::TextureView, postprocess_source: PostProcessSource) {
        // temporal dither: while enabled, advance the frame counter and re-
        // randomize the noise lattice every frame; disabled it costs nothing
        if self.params_uniform.temporal > 0.0 {
            self.frame_index = (self.frame_index + 1.0) % 1024.0;
            self.params_uniform.frame = self.frame_index;

            let s = Services::get();
            let queue = s.gpu.queue();
            queue.write_buffer(&self.params, 0, cast_slice(&[self.params_uniform]));
        }

        // pick the blit bind group for the chain's current source: the
        // downsample below reads it, everything else on the guide grid is
        // resolution-fixed and prebuilt
        let blit_bg: &BindGroup;
        match postprocess_source {
            PostProcessSource::Offscreen => blit_bg = &self.blit_bg_input,
            PostProcessSource::ViewA => blit_bg = &self.blit_bg_a,
            PostProcessSource::ViewB => blit_bg = &self.blit_bg_b,
        }

        // the whole guide chain, on the half grid between the two blits:
        // source DOWN -> sobel -> blur h -> blur v -> kernel -> guide UP
        // into the full-res chain target, so downstream features and the
        // ping-pong contract never see a different resolution
        for (pipeline, bg, target) in [
            (&self.blit_pipeline, blit_bg, &self.guide_view_src),
            (&self.flow_pipeline, &self.flow_bg_sobel, &self.flow_view_src),
            (&self.blur_pipeline, &self.flow_bg_h, &self.flow_view_tmp),
            (&self.blur_pipeline, &self.flow_bg_v, &self.flow_view_src),
            (&self.pipeline, &self.bind_group_kernel, &self.guide_view_dst),
            (&self.blit_pipeline, &self.blit_bg_guide, output_view),
        ] {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("kuwahara guide chain pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
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

            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, bg, &[]);
            pass.draw(0..3, 0..1);
        }
    }
}
