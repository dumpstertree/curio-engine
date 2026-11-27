use bytemuck::{self, cast_slice};
use curio_core::{ASSET_UID_TEXTURE_FONT_ATLAS, Assets, GraphicsMapping, Ledger, Services, Severity, TextureAsset, Vector2};
use egui_wgpu::wgpu::{
    self, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendState, Buffer, BufferAddress, BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites, CommandEncoder, FragmentState, LoadOp,
    MultisampleState, Operations, PipelineLayout, PipelineLayoutDescriptor, PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, SamplerBindingType, ShaderModuleDescriptor, ShaderSource, ShaderStages, StoreOp, TextureFormat, TextureSampleType,
    TextureView, TextureViewDimension, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};
use std::sync::Arc;

use crate::{DEFAULT_FONT, SysRecordUi, UiFontDesc, UiFontGrid, UiKind, UiScreen};

/// A painted corner of a UI quad, in normalized device coordinates.
///
/// The pass converts screen pixels to NDC on the CPU, so the shader only
/// has to pick a texel (for glyphs) or pass the tint through (for boxes).
/// repr(C) packs the fields tightly — position (8 bytes) plus uv (8 bytes)
/// puts the tint at byte 16, exactly where desc() reads it. this used to
/// need two filler words because the buffer layout read the tint at 16
/// while repr(C) had put it at 12: the GPU read [g, b, a, pad], alpha came
/// out of the padding as 0, and every quad blended fully transparent with
/// no validation error. the uv pair now occupies that gap and keeps the
/// stride at 32
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub struct UiVertex {
    pub position: [f32; 2],
    // texture coords for this corner. flat boxes carry (-1, -1): the
    // fragment keeps the pure tint for those. glyph quads carry the
    // shared font atlas cell, v growing downward like UI y
    pub uv: [f32; 2],
    pub tint: [f32; 4],
}
impl UiVertex {
    pub const fn new(position: [f32; 2], uv: [f32; 2], tint: [f32; 4]) -> UiVertex {
        UiVertex { position, uv, tint }
    }
    pub fn desc() -> VertexBufferLayout<'static> {
        egui_wgpu::wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<UiVertex>() as BufferAddress,
            step_mode: egui_wgpu::wgpu::VertexStepMode::Vertex,
            attributes: &[
                egui_wgpu::wgpu::VertexAttribute {
                    // position (already NDC)
                    offset: std::mem::size_of::<[f32; 0]>() as BufferAddress,
                    shader_location: 0,
                    format: egui_wgpu::wgpu::VertexFormat::Float32x2,
                },
                egui_wgpu::wgpu::VertexAttribute {
                    // uv at byte 8
                    offset: std::mem::size_of::<[f32; 2]>() as BufferAddress,
                    shader_location: 1,
                    format: egui_wgpu::wgpu::VertexFormat::Float32x2,
                },
                egui_wgpu::wgpu::VertexAttribute {
                    // tint at byte 16 — position (8) plus uv (8)
                    offset: std::mem::size_of::<[f32; 4]>() as BufferAddress,
                    shader_location: 2,
                    format: egui_wgpu::wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// Draws every screen's UI boxes and text labels onto the swapchain after
/// post-processing.
///
/// This is not egui: the layout habit already resolved each box in screen
/// pixels, so the pass is one dynamic vertex buffer and one tiny shader.
/// Text quads sample the same built-in font atlas 3D text samples — only
/// the asset and the default.font grid convention are shared, not the
/// mesh/material plumbing. Each screen lands inside its GraphicsMapping
/// rect on the swapchain.
pub struct UiOverlayPass {
    // built on the first frame that has anything to draw
    pipeline: Option<RenderPipeline>,
    vertex_buffer: Option<Buffer>,
    vertex_capacity: usize,
    // the shared atlas: curio_core's built-in texture asset, held as an
    // Arc so the GPU texture stays alive while the bind group borrows its
    // view. the grid is parsed from the same default.font 3D text reads
    font_atlas: Option<Arc<TextureAsset>>,
    font_grid: Option<UiFontGrid>,
    font_layout: Option<BindGroupLayout>,
    font_bind_group: Option<BindGroup>,
    // the pipeline references this by handle — it must outlive the frame
    pipeline_layout: Option<PipelineLayout>,
}

impl UiOverlayPass {
    pub fn new() -> UiOverlayPass {
        UiOverlayPass {
            pipeline: None,
            vertex_buffer: None,
            vertex_capacity: 0,
            font_atlas: None,
            font_grid: None,
            font_layout: None,
            font_bind_group: None,
            pipeline_layout: None,
        }
    }

    pub fn draw_ui_overlays(&mut self, ledger: &mut Vec<Ledger>, encoder: &mut CommandEncoder, output_view: &TextureView, mappings: &[GraphicsMapping]) {
        // gather every screen's boxes into one vertex list
        let mut vertices: Vec<UiVertex> = Vec::new();
        let gpu = &Services::get().gpu;
        let capture = Vector2::new(gpu.capture_width as f32, gpu.capture_height as f32);
        let screens = ledger.len().min(mappings.len());
        for screen_index in 0..screens {
            // keep the record's mapping the one this pass paints with, for SysRecordUi::screen()
            let mapping = &mappings[screen_index];
            let recorded = ledger[screen_index].read::<SysRecordUi>().mapping.clone();
            if recorded.viewport_min != mapping.viewport_min || recorded.viewport_max != mapping.viewport_max {
                ledger[screen_index].write::<SysRecordUi>(|state_ui| state_ui.mapping = mapping.clone());
            }
            let state_ui = ledger[screen_index].read::<SysRecordUi>();
            if state_ui.draw_calls.is_empty() {
                continue;
            }
            let mut screen = UiScreen::new(state_ui.viewport, mapping, capture);
            if !screen.is_valid() {
                // the habit has not run on this screen yet
                screen.viewport = screen.pixels();
                if !screen.is_valid() {
                    continue;
                }
            }

            for draw_call in state_ui.draw_calls.iter() {
                let rect = draw_call.rect;
                if !(rect.width > 0.0 && rect.height > 0.0) || draw_call.scale.x == 0.0 || draw_call.scale.y == 0.0 {
                    continue;
                }
                let color = draw_call.color;
                // curio_core Color keeps its channels private behind accessors
                let tint = [color.as_r_01(), color.as_g_01(), color.as_b_01(), color.as_a_01()];

                // a point of the upright box → turned and scaled about its centre → NDC
                let place = |px: f32, py: f32| {
                    let turned = rect.turn_point(px, py, draw_call.rotation, draw_call.scale);
                    screen.to_ndc(turned.x, turned.y)
                };
                let top_left = place(rect.x, rect.y);
                let top_right = place(rect.x + rect.width, rect.y);
                let bottom_left = place(rect.x, rect.y + rect.height);
                let bottom_right = place(rect.x + rect.width, rect.y + rect.height);

                // a Text node with a label paints glyphs, not a box; an
                // empty Text node still paints its placeholder quad
                if !(draw_call.kind == UiKind::Text && !draw_call.text.is_empty()) {
                    // one quad, two triangles — wound counter-clockwise in
                    // NDC. the flat uv marks the fragment's no-texture path
                    let flat_uv = [-1.0, -1.0];
                    vertices.push(UiVertex::new(bottom_left, flat_uv, tint));
                    vertices.push(UiVertex::new(bottom_right, flat_uv, tint));
                    vertices.push(UiVertex::new(top_left, flat_uv, tint));
                    vertices.push(UiVertex::new(top_left, flat_uv, tint));
                    vertices.push(UiVertex::new(bottom_right, flat_uv, tint));
                    vertices.push(UiVertex::new(top_right, flat_uv, tint));
                }

                // glyphs: monospace, one advance per character, centred in
                // the box and clipped to its right edge. cells come from the
                // shared atlas grid, exactly as 3D text reads them
                if !draw_call.text.is_empty() && draw_call.font_size > 0.0 {
                    match &self.font_grid {
                        Some(grid) => {
                            let size = draw_call.font_size;
                            let line_width = (draw_call.text.chars().count() as f32) * size;
                            let pen_y = if size > rect.height { rect.y } else { rect.y + (rect.height - size) * 0.5 };
                            let mut pen_x = if line_width > rect.width { rect.x } else { rect.x + (rect.width - line_width) * 0.5 };
                            for ch in draw_call.text.chars() {
                                if ch == ' ' {
                                    pen_x += size;
                                    continue;
                                }
                                if pen_x + size > rect.x + rect.width {
                                    break;
                                }
                                match grid.glyph_uv(ch) {
                                    Some([u_min, v_top, u_max, v_bottom]) => {
                                        // cells sit in the upright box, so they lean with it
                                        let cell_top_left = place(pen_x, pen_y);
                                        let cell_top_right = place(pen_x + size, pen_y);
                                        let cell_bottom_left = place(pen_x, pen_y + size);
                                        let cell_bottom_right = place(pen_x + size, pen_y + size);
                                        // same winding as the box quads.
                                        // the cell's top edge is v_top since
                                        // v grows downward in the texture
                                        vertices.push(UiVertex::new(cell_bottom_left, [u_min, v_bottom], tint));
                                        vertices.push(UiVertex::new(cell_bottom_right, [u_max, v_bottom], tint));
                                        vertices.push(UiVertex::new(cell_top_left, [u_min, v_top], tint));
                                        vertices.push(UiVertex::new(cell_top_left, [u_min, v_top], tint));
                                        vertices.push(UiVertex::new(cell_bottom_right, [u_max, v_bottom], tint));
                                        vertices.push(UiVertex::new(cell_top_right, [u_max, v_top], tint));
                                    }
                                    None => {}
                                }
                                pen_x += size;
                            }
                        }
                        None => {}
                    }
                }
            }
        }
        if vertices.is_empty() {
            return;
        }

        let device = Services::get().gpu.device();
        let queue = Services::get().gpu.queue();

        // lazy pipeline — first frame with boxes builds it
        if self.pipeline.is_none() {
            // the atlas is loaded once and cached on the pass. load_asset
            // hands back an Arc, so the pass keeps the Arc while bind group
            // entries borrow the view/sampler inside it
            let atlas = Assets::load_asset::<TextureAsset>(&ASSET_UID_TEXTURE_FONT_ATLAS);
            match UiFontDesc::parse(DEFAULT_FONT) {
                Some(desc) => {
                    self.font_grid = Some(UiFontGrid::new(desc, atlas.texture.width(), atlas.texture.height()));
                }
                None => {
                    if ledger.len() > 0 {
                        ledger[0].log(Severity::Warning, &"ui_overlay: default.font could not be parsed — text labels will be skipped");
                    }
                }
            }

            let font_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("ui font atlas layout"),
                entries: &[
                    egui_wgpu::wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            multisampled: false,
                            view_dimension: TextureViewDimension::D2,
                            sample_type: TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    egui_wgpu::wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
            self.font_bind_group = Some(device.create_bind_group(&BindGroupDescriptor {
                label: Some("ui font atlas"),
                layout: &font_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&atlas.view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&atlas.sampler),
                    },
                ],
            }));

            // the pass keeps the Arc alive so the GPU texture/sampler stay
            // valid while the bind group borrows the view inside it
            self.font_atlas = Some(atlas);

            // pinned wgpu wants a PipelineLayout wrapping the bind group
            // layouts, not the bind group layout itself
            let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("ui overlay layout"),
                bind_group_layouts: &[&font_layout],
                push_constant_ranges: &[],
            });

            // WebGPU's front face is defined in y-down framebuffer space,
            // which is easy to get backwards for CPU-built NDC quads — UI
            // quads are single-layer and flat, so cull nothing
            let mut ui_primitive: PrimitiveState = PrimitiveState::default();
            ui_primitive.cull_mode = None;

            let shader = device.create_shader_module(ShaderModuleDescriptor {
                label: Some("ui overlay shader"),
                source: ShaderSource::Wgsl(include_str!("ui_overlay.wgsl").into()),
            });
            self.pipeline = Some(device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("ui overlay pipeline"),
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[UiVertex::desc()],
                    compilation_options: Default::default(),
                },
                fragment: Some(FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(ColorTargetState {
                        format: TextureFormat::Rgba8UnormSrgb,
                        blend: Some(BlendState::ALPHA_BLENDING),
                        write_mask: ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: ui_primitive,
                depth_stencil: None,
                multisample: MultisampleState::default(),
                multiview: None,
                cache: None,
            }));
            // the pipeline and bind group reference these layouts by handle,
            // so the pass has to keep all three alive
            self.font_layout = Some(font_layout);
            self.pipeline_layout = Some(pipeline_layout);
        }

        // one dynamic vertex buffer, grown like the mesh instance buffer
        if self.vertex_capacity < vertices.len() {
            self.vertex_buffer = Some(device.create_buffer(&BufferDescriptor {
                label: Some("ui vertices"),
                size: (vertices.len() * std::mem::size_of::<UiVertex>()) as u64,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
            self.vertex_capacity = vertices.len();
        }
        queue.write_buffer(self.vertex_buffer.as_ref().unwrap(), 0, bytemuck::cast_slice(&vertices));

        // load what post-process painted, blend the boxes over it
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("ui overlay pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: output_view,
                resolve_target: None,
                ops: Operations { load: LoadOp::Load, store: StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(self.pipeline.as_ref().unwrap());
        pass.set_bind_group(0, self.font_bind_group.as_ref().unwrap(), &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap().slice(..));
        pass.draw(0..(vertices.len() as u32), 0..1);
    }
}
