mod egui_app_state;
mod egui_tools;
use crate::egui_tools::EguiRenderer;
use core::collections::camera_uniform::CameraUniform;
use core::collections::event_queue::EventQueue2;
use core::collections::game_state::GameState;
use core::collections::gizmo::Gizmo;
use core::collections::material::Material;
use core::collections::matrix4x4::Matrix4x4;
use core::collections::{draw_call::DrawCall, mesh::Vertex};
use core::io::asset_loader::AssetLoader;
use core::io::texture_asset::TextureAsset;
use core::system::system_component::SystemComponent;
use core::system::system_components::system_component_graphics::SystemComponentGraphics;
use core::system::system_game_states::state_camera::CameraState;
use core::system::system_game_states::state_debug::StateDebug;
use core::system::system_game_states::state_draw::DrawCallsState;
use core::system::system_game_states::state_gizmos::GizmosState;
use core::system::system_game_states::state_gui::GUIState;
use core::system::system_game_states::state_gui_debug::GUIStateDebug;
use core::system_adapters::adapter_system_gpu::SystemGPU;

use egui::{Color32, Frame, Pos2, Ui};
use egui_wgpu::ScreenDescriptor;
use egui_wgpu::wgpu::{
    self, BindGroup, BindGroupLayout, BlendState, ColorTargetState, Device, FragmentState, RenderPass, Surface, SurfaceConfiguration,
};
use egui_wgpu::wgpu::{
    Buffer, CommandEncoder, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPipeline, ShaderModule, SurfaceTexture, TextureView,
    util::DeviceExt,
};
use std::iter;
use winit::event::WindowEvent;

pub struct SystemComponentDefaultGraphics {
    camera_rendereing: CameraRenderingComponents,
    // projection: Projection,
    egui_renderer: EguiRenderer,
}

impl SystemComponentDefaultGraphics {
    fn draw_gizmos(&mut self, draw_call: &Gizmo, config: &SurfaceConfiguration, device: &Device, render_pass: &mut RenderPass) {
        // iterate over each mesh in the draw call
        //
        let mesh = &draw_call.mesh;
        let mut material = Material::new(AssetLoader::load_shader_desc("assets/shader/gizmo.shader"));

        material.set_color_with_label(draw_call.color.clone(), "color");
        // create material bind group
        // let diffuse_bind_group = egui_wgpu::wgpuGraphicsComponent::get_diffuse_binding(&state, &material.textures[..]);
        let diffuse_bind_group = material.get_texture_binding_group(device);
        let color_bind_group = material.get_color_binding_group(device);
        // set render pipeline
        let rp = SystemComponentDefaultGraphics::get_render_pipeline(
            &self.camera_rendereing.camera_bind_group_layout,
            config,
            device,
            material.shader.clone(),
            &diffuse_bind_group.1,
            &color_bind_group.1,
            true,
        );
        render_pass.set_pipeline(&rp);

        // create the instance buffer
        let n_buffer = device.create_buffer_init(&egui_wgpu::wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&draw_call.matrix),
            usage: egui_wgpu::wgpu::BufferUsages::VERTEX,
        });
        // fetch the cached buffers for the mesh
        // let buffers: (&Buffer, &Buffer) = self.buffer_cache.get_vertex_buffer(device, mesh);
        let buffers = (mesh.get_vertex_buffer_for_device(device), mesh.get_index_buffer_for_device(device));

        // set buffers
        render_pass.set_vertex_buffer(0, buffers.0.slice(..));
        render_pass.set_vertex_buffer(1, n_buffer.slice(..));
        render_pass.set_index_buffer(buffers.1.slice(..), egui_wgpu::wgpu::IndexFormat::Uint32);

        // set bind groups
        render_pass.set_bind_group(0, &diffuse_bind_group.0, &[]);
        render_pass.set_bind_group(1, &self.camera_rendereing.camera_bind_group, &[]);
        render_pass.set_bind_group(2, &color_bind_group.0, &[]);

        // draw
        render_pass.draw_indexed(0..(mesh.indicies.len() as u32), 0, 0..draw_call.matrix.len() as u32);
    }
    fn draw_draw_call(&mut self, draw_call: &DrawCall, config: &SurfaceConfiguration, device: &Device, render_pass: &mut RenderPass) {
        // iterate over each mesh in the draw call
        for i in 0..draw_call.mesh.len() {
            //
            let mesh = &draw_call.mesh[i];
            let material = &draw_call.materials[i];
            // create material bind group
            // let diffuse_bind_group = egui_wgpu::wgpuGraphicsComponent::get_diffuse_binding(&state, &material.textures[..]);
            let diffuse_bind_group = material.get_texture_binding_group(device);
            let color_bind_group = material.get_color_binding_group(device);
            // set render pipeline
            let rp = SystemComponentDefaultGraphics::get_render_pipeline(
                &self.camera_rendereing.camera_bind_group_layout,
                config,
                device,
                material.shader.clone(),
                &diffuse_bind_group.1,
                &color_bind_group.1,
                false,
            );
            render_pass.set_pipeline(&rp);

            // create the instance buffer
            let n_buffer = device.create_buffer_init(&egui_wgpu::wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&draw_call.matrix),
                usage: egui_wgpu::wgpu::BufferUsages::VERTEX,
            });
            // fetch the cached buffers for the mesh
            // let buffers: (&Buffer, &Buffer) = self.buffer_cache.get_vertex_buffer(device, mesh);
            let buffers = (mesh.get_vertex_buffer_for_device(device), mesh.get_index_buffer_for_device(device));

            // set buffers
            render_pass.set_vertex_buffer(0, buffers.0.slice(..));
            render_pass.set_vertex_buffer(1, n_buffer.slice(..));
            render_pass.set_index_buffer(buffers.1.slice(..), egui_wgpu::wgpu::IndexFormat::Uint32);

            // set bind groups
            render_pass.set_bind_group(0, &diffuse_bind_group.0, &[]);
            render_pass.set_bind_group(1, &self.camera_rendereing.camera_bind_group, &[]);
            render_pass.set_bind_group(2, &color_bind_group.0, &[]);

            // draw
            render_pass.draw_indexed(0..(mesh.indicies.len() as u32), 0, 0..draw_call.matrix.len() as u32);
        }
    }
}
impl SystemComponent for SystemComponentDefaultGraphics {
    fn order(&self) -> i32 {
        9000
    }
    fn init(&mut self, gs: &mut GameState) {
        let cs = gs.get_value2::<CameraState>();
        self.camera_rendereing = CameraRenderingComponents::new(cs.get_uniform());
    }
    fn tick(&mut self, game_state: &mut GameState, system_event_queue: &mut EventQueue2) {
        let surface = &SystemGPU::get_surface();
        let device = &SystemGPU::get_device();
        let depth = &SystemGPU::get_depth_texture();
        let window = &SystemGPU::get_window();
        let queue = &SystemGPU::get_queue();
        let config = &&SystemGPU::get_config();

        // get gamestate data
        let state_camera = game_state.get_value2::<CameraState>();
        let state_draws = game_state.get_value2::<DrawCallsState>();
        let state_gizmos = game_state.get_value2::<GizmosState>();
        let state_gui = &game_state.get_value2::<GUIState>();
        let state_gui_debug = &game_state.get_value2::<GUIStateDebug>();

        //
        let output = SystemComponentDefaultGraphics::get_output_texture(surface);
        let mut encoder = SystemComponentDefaultGraphics::get_encoder(device);
        {
            let view = output
                .texture
                .create_view(&egui_wgpu::wgpu::TextureViewDescriptor::default());
            let mut render_pass = encoder.begin_render_pass(&egui_wgpu::wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[SystemComponentDefaultGraphics::get_color_atatchment(&view)],
                depth_stencil_attachment: SystemComponentDefaultGraphics::get_depth_attatchment(depth),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            queue.write_buffer(
                &self.camera_rendereing.camera_buffer,
                0,
                bytemuck::cast_slice(&[state_camera.get_uniform()]),
            );

            // get all draw calls from state
            let draw_calls = &state_draws.draw_calls;
            for draw_call in draw_calls {
                self.draw_draw_call(draw_call, config, device, &mut render_pass);
            }

            let gizmos_calls = &state_gizmos.draw_calls;
            for gizmo in gizmos_calls {
                self.draw_gizmos(gizmo, config, device, &mut render_pass);
            }
        }

        // start gui
        self.egui_renderer.begin_frame(window);

        for gui_window in &state_gui.guis {
            let pos = gui_window.position;
            let mut x = |ui: &mut Ui| {
                // let response = ui.allocate_response(ui.available_size(), Sense::click());
                for element in &gui_window.children {
                    match &element.gui_type {
                        core::system::system_game_states::state_gui::GuiElementTypes::Rectangle => todo!(),
                        core::system::system_game_states::state_gui::GuiElementTypes::Ellipse => todo!(),
                        core::system::system_game_states::state_gui::GuiElementTypes::Label(label_desc) => {
                            for (_text_style, font_id) in ui.style_mut().text_styles.iter_mut() {
                                font_id.size = label_desc.font_size // whatever size you want here
                            }
                            ui.colored_label(
                                Color32::from_rgb(label_desc.color.r_0255(), label_desc.color.g_0255(), label_desc.color.b_0255()),
                                &label_desc.contents,
                            );
                        }
                        core::system::system_game_states::state_gui::GuiElementTypes::Button(button_desc) => {
                            let b = ui.button(&button_desc.contents);
                            if b.clicked() {
                                (button_desc.on_click)(game_state, system_event_queue);
                            }
                            if b.hovered() {}
                        }
                    };
                }
            };
            egui::Window::new(gui_window.instance_id.clone())
                .frame(Frame::new().fill(Color32::TRANSPARENT))
                .title_bar(false)
                .resizable(false)
                .current_pos(Pos2::new(pos.x, pos.y))
                .show(self.egui_renderer.context(), &mut x);
        }

        if game_state.get_value2::<StateDebug>().is_inspecting {
            let gui_window = &state_gui_debug.finalize(game_state);
            let mut x = |ui: &mut Ui| {
                for element in &gui_window.children {
                    match &element.gui_type {
                        core::system::system_game_states::state_gui::GuiElementTypes::Rectangle => todo!(),
                        core::system::system_game_states::state_gui::GuiElementTypes::Ellipse => todo!(),
                        core::system::system_game_states::state_gui::GuiElementTypes::Label(label_desc) => {
                            for (_text_style, font_id) in ui.style_mut().text_styles.iter_mut() {
                                font_id.size = label_desc.font_size // whatever size you want here
                            }
                            ui.colored_label(
                                Color32::from_rgb(label_desc.color.r_0255(), label_desc.color.g_0255(), label_desc.color.b_0255()),
                                &label_desc.contents,
                            );
                        }
                        core::system::system_game_states::state_gui::GuiElementTypes::Button(button_desc) => {
                            let b = ui.button(&button_desc.contents);
                            if b.clicked() {
                                (button_desc.on_click)(game_state, system_event_queue);
                            }
                            if b.hovered() {}
                        }
                    };
                }
            };
            egui::Window::new(gui_window.instance_id.clone())
                .frame(Frame::new().fill(Color32::TRANSPARENT))
                .title_bar(false)
                .current_pos(Pos2::new(gui_window.position.x, gui_window.position.y))
                .show(self.egui_renderer.context(), &mut x);
        }

        let surface_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [config.width, config.height],
            pixels_per_point: 1.0, //window.as_ref().scale_factor() as f32 * scale_factor as f32,
        };

        self.egui_renderer
            .end_frame_and_draw(device, queue, &mut encoder, window, &surface_view, screen_descriptor);

        // submit commands for execution
        queue.submit(iter::once(encoder.finish()));
        // present the completed texture
        output.present();

        // clear state
        game_state.edit::<GizmosState>(|x| {
            x.draw_calls.clear();
        });
        game_state.edit::<DrawCallsState>(|x| {
            x.draw_calls.clear();
        });
        game_state.edit::<GUIState>(|x| {
            x.guis.clear();
        });
        game_state.edit::<GUIStateDebug>(|x| {
            x.clear();
        });

        // return no changes
    }
    fn raw_event(&mut self, event: WindowEvent) {
        let window = SystemGPU::get_window();
        self.egui_renderer.handle_input(&window, &event);
    }
}

impl SystemComponentGraphics for SystemComponentDefaultGraphics {}
impl SystemComponentDefaultGraphics {
    pub fn new() -> Box<SystemComponentDefaultGraphics> {
        // let p = Projection::new(1920, 1080, cgmath::Deg(45.0), 0.1, 100.0);
        let c = SystemGPU::get_config();
        let w = &(*SystemGPU::get_window());
        let d = &(*SystemGPU::get_device());

        Box::new(SystemComponentDefaultGraphics {
            camera_rendereing: CameraRenderingComponents::new(CameraUniform::new()),
            // projection: p,
            egui_renderer: EguiRenderer::new(d, c.format, None, 1, w),
        })
    }

    fn get_render_pipeline(
        camera_bind: &BindGroupLayout,
        config: &SurfaceConfiguration,
        device: &Device,
        shader: ShaderModule,
        texture_bind_group_layout: &BindGroupLayout,
        color_bind_group_layout: &BindGroupLayout,
        wireframe: bool,
    ) -> RenderPipeline {
        let render_pipeline_layout = device.create_pipeline_layout(&egui_wgpu::wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &camera_bind, &color_bind_group_layout],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&egui_wgpu::wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: egui_wgpu::wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                // buffers: &[super::model::ModelVertex::desc(), InstanceRaw::desc()],
                buffers: &[Vertex::desc(), Matrix4x4::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState {
                        color: egui_wgpu::wgpu::BlendComponent::REPLACE,
                        alpha: egui_wgpu::wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: egui_wgpu::wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: egui_wgpu::wgpu::PrimitiveState {
                topology: egui_wgpu::wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: egui_wgpu::wgpu::FrontFace::Ccw,
                cull_mode: Some(egui_wgpu::wgpu::Face::Back),
                polygon_mode: if wireframe {
                    egui_wgpu::wgpu::PolygonMode::Line
                } else {
                    egui_wgpu::wgpu::PolygonMode::Fill
                },
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(egui_wgpu::wgpu::DepthStencilState {
                format: TextureAsset::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: egui_wgpu::wgpu::CompareFunction::Less,
                stencil: egui_wgpu::wgpu::StencilState::default(),
                bias: egui_wgpu::wgpu::DepthBiasState::default(),
            }),
            // depth_stencil: None,
            multisample: egui_wgpu::wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
            // Useful for optimizing shader compilation on Android
            cache: None,
        })
    }

    fn get_output_texture(surface: &Surface) -> SurfaceTexture {
        surface.get_current_texture().unwrap()
    }
    fn get_encoder(device: &Device) -> CommandEncoder {
        device.create_command_encoder(&egui_wgpu::wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        })
    }
    fn get_color_atatchment<'a>(view: &'a TextureView) -> Option<RenderPassColorAttachment<'a>> {
        Some(egui_wgpu::wgpu::RenderPassColorAttachment {
            view: view,
            resolve_target: None,
            ops: egui_wgpu::wgpu::Operations {
                load: egui_wgpu::wgpu::LoadOp::Clear(egui_wgpu::wgpu::Color {
                    r: 0.1,
                    g: 0.2,
                    b: 0.3,
                    a: 1.0,
                }),
                store: egui_wgpu::wgpu::StoreOp::Store,
            },
        })
    }
    fn get_depth_attatchment<'a>(depth: &'a TextureAsset) -> Option<RenderPassDepthStencilAttachment<'a>> {
        Some(egui_wgpu::wgpu::RenderPassDepthStencilAttachment {
            view: &depth.view,
            depth_ops: Some(egui_wgpu::wgpu::Operations {
                load: egui_wgpu::wgpu::LoadOp::Clear(1.0),
                store: egui_wgpu::wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        })
    }
}

// #[derive(Clone)]
// pub struct DrawCallsState {
//     pub draw_calls: Vec<DrawCall>,
// }
// impl DrawCallsState {
//     pub fn new<'a>() -> DrawCallsState {
//         DrawCallsState { draw_calls: Vec::new() }
//     }
// }

pub struct CameraRenderingComponents {
    camera_bind_group: BindGroup,
    camera_bind_group_layout: BindGroupLayout,
    // camera_uniform: CameraUniform,
    camera_buffer: Buffer,
}
impl CameraRenderingComponents {
    // pub fn update_view_matrix(&mut self, state_camera: CameraState, projection: &Projection) {
    //     self.camera_uniform
    //         .update_view_proj(&state_camera, &projection)
    // }

    pub fn new(camera_uniform: CameraUniform) -> CameraRenderingComponents {
        let device = SystemGPU::get_device();
        // let camera = CameraState::default();
        // let mut camera_uniform: CameraUniform = CameraUniform::new();
        // camera_uniform.update_view_proj(&camera, projection);

        let camera_buffer = device.create_buffer_init(&egui_wgpu::wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            // contents: bytemuck::cast_slice(&[camera_uniform]),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: egui_wgpu::wgpu::BufferUsages::UNIFORM | egui_wgpu::wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout = device.create_bind_group_layout(&egui_wgpu::wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: egui_wgpu::wgpu::ShaderStages::VERTEX,
                ty: egui_wgpu::wgpu::BindingType::Buffer {
                    ty: egui_wgpu::wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        });

        let camera_bind_group = device.create_bind_group(&egui_wgpu::wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        CameraRenderingComponents {
            camera_bind_group,
            camera_bind_group_layout,
            // camera_uniform,
            camera_buffer,
        }
    }
}
