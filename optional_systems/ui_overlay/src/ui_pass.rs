use curio_core::{GraphicsMapping, Ledger, Services, Severity, Vector2};
use egui_wgpu::wgpu::{self, Buffer, BufferAddress, BufferDescriptor, BufferUsages, BlendState, Color, CommandEncoder, ColorTargetState, ColorWrites, FragmentState, LoadOp, Operations, MultisampleState, PrimitiveState, RenderPass, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, VertexState, ShaderSource, StoreOp, TextureFormat, TextureView, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};
use bytemuck::{self, cast_slice};

use crate::{SysRecordUi, UiColor, UiDrawCall, UiKind, UiRect};

/// A painted corner of a UI quad, in normalized device coordinates.
///
/// The pass converts screen pixels to NDC on the CPU, so the shader only
/// has to pass the tint through.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub struct UiVertex {
    pub position: [f32; 2],
    // Float32x4 needs 16-byte alignment — keep the tint on its own word
    pub pad: f32,
    pub tint: [f32; 4],
    // tint is 16 bytes at offset 16, so the stride is a third word short
    pub pad2: f32,
}
impl UiVertex {
    pub const fn new(position: [f32; 2], tint: [f32; 4]) -> UiVertex {
        UiVertex { position, pad: 0.0, tint, pad2: 0.0 }
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
                    // tint. offset 16 — Float32x4 must be 16-byte aligned
                    offset: std::mem::size_of::<[f32; 4]>() as BufferAddress,
                    shader_location: 1,
                    format: egui_wgpu::wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// Draws every screen's UI boxes onto the swapchain after post-processing.
///
/// This is not egui: the layout habit already resolved each box in screen
/// pixels, so the pass is one dynamic vertex buffer and one tiny shader.
/// Each screen lands inside its GraphicsMapping rect on the swapchain.
pub struct UiOverlayPass {
    // built on the first frame that has anything to draw
    pipeline: Option<RenderPipeline>,
    vertex_buffer: Option<Buffer>,
    vertex_capacity: usize,
}

impl UiOverlayPass {
    pub fn new() -> UiOverlayPass {
        UiOverlayPass { pipeline: None, vertex_buffer: None, vertex_capacity: 0 }
    }

    pub fn draw_ui_overlays(&mut self, ledger: &mut Vec<Ledger>, encoder: &mut CommandEncoder, output_view: &TextureView, mappings: &[GraphicsMapping]) {
        // gather every screen's boxes into one vertex list
        let mut vertices: Vec<UiVertex> = Vec::new();
        let gpu = &Services::get().gpu;
        let screens = ledger.len().min(mappings.len());
        for screen_index in 0..screens {
            let state_ui = ledger[screen_index].read::<SysRecordUi>();
            if state_ui.draw_calls.is_empty() {
                continue;
            }
            // the viewport the layout habit sized against, same fallback
            // order as in the habit
            let mut viewport = state_ui.viewport;
            if !(viewport.x > 0.0 && viewport.y > 0.0) {
                viewport = Vector2::new(gpu.capture_width as f32, gpu.capture_height as f32);
            }
            // this screen's rect on the swapchain. UI y grows down, GPU y
            // grows up — so the UI's top edge is the mapping's max.y
            let mapping = &mappings[screen_index];
            let min_x = mapping.viewport_min.x;
            let max_x = mapping.viewport_max.x;
            let min_y = mapping.viewport_min.y;
            let max_y = mapping.viewport_max.y;

            for draw_call in state_ui.draw_calls.iter() {
                let rect = draw_call.rect;
                if !(rect.width > 0.0 && rect.height > 0.0) {
                    continue;
                }
                let color = draw_call.color;
                let tint = [color.r, color.g, color.b, color.a];

                // UI pixels → the screen's share of the swapchain → NDC.
                // left/right, then the y flip for the UI's down axis
                let left = min_x + rect.x / viewport.x * (max_x - min_x);
                let right = min_x + (rect.x + rect.width) / viewport.x * (max_x - min_x);
                let top = max_y - rect.y / viewport.y * (max_y - min_y);
                let bottom = max_y - (rect.y + rect.height) / viewport.y * (max_y - min_y);
                let x0 = 2.0 * left - 1.0;
                let x1 = 2.0 * right - 1.0;
                let y0 = 2.0 * bottom - 1.0;
                let y1 = 2.0 * top - 1.0;

                // one quad, two triangles — wound counter-clockwise in NDC,
                // which is wgpu's front face under the default cull mode
                vertices.push(UiVertex::new([x0, y0], tint));
                vertices.push(UiVertex::new([x1, y0], tint));
                vertices.push(UiVertex::new([x0, y1], tint));
                vertices.push(UiVertex::new([x0, y1], tint));
                vertices.push(UiVertex::new([x1, y0], tint));
                vertices.push(UiVertex::new([x1, y1], tint));
            }
        }
        if vertices.is_empty() {
            return;
        }

        // diagnostic while wiring is unconfirmed: what actually reaches the
        // swapchain. drop this once the overlay paints
        match ledger.get(0) {
            Some(screen) => {
                let first = vertices[0];
                let mut min_x: f32 = 0.0;
                let mut max_x: f32 = 0.0;
                if mappings.len() > 0 {
                    min_x = mappings[0].viewport_min.x;
                    max_x = mappings[0].viewport_max.x;
                }
                screen.log(
                    Severity::Info,
                    &format!(
                        "ui pass: {} vertices; first ndc x {} y {}; map x {}..{}",
                        vertices.len(),
                        first.position[0],
                        first.position[1],
                        min_x,
                        max_x,
                    ),
                );
            }
            None => {}
        }

        // TARGET TEST (temporary): a no-draw red clear of output_view. If the
        // window turns red, this encoder+view does reach presentation, and an
        // invisible pipeline means a pipeline/buffer problem; if nothing
        // changes, output_view is not what gets shown.
        let debug_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("ui target test"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: output_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::RED),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        return;

        let device = Services::get().gpu.device();
        let queue = Services::get().gpu.queue();

        // lazy pipeline — first frame with boxes builds it
        if self.pipeline.is_none() {
            let shader = device.create_shader_module(ShaderModuleDescriptor {
                label: Some("ui overlay shader"),
                source: ShaderSource::Wgsl(include_str!("ui_overlay.wgsl").into()),
            });
            self.pipeline = Some(device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("ui overlay pipeline"),
                // the shader reads only vertex attributes — no bindings
                layout: None,
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
                // TriangleList, no culling — exactly what the quads need
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                multiview: None,
                cache: None,
            }));
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
        queue.write_buffer(
            self.vertex_buffer.as_ref().unwrap(),
            0,
            bytemuck::cast_slice(&vertices),
        );

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
        pass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap().slice(..));
        pass.draw(0..(vertices.len() as u32), 0..1);
    }
}
