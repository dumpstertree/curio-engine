// use std::os::linux::raw::stat;
// use std::{iter, sync::Arc};

// use super::super::Collections::DrawCall::DrawCall;
// use crate::Collections::matrix4x4::{self, Matrix4x4};
// use crate::Collections::Mesh::Vertex;
// use crate::Window::state::State;
// use crate::Window::CameraState::CameraState;
// use crate::IO::texture_asset::{self, Texture_asset};
// use wgpu::SurfaceError::Lost;
// use wgpu::SurfaceError::Outdated;
// use wgpu::{util::DeviceExt, Buffer, CommandEncoder, RenderPassColorAttachment, RenderPassDepthStencilAttachment, SurfaceTexture, TextureView};
// use wgpu::{BindGroup, BindGroupLayout, RenderPass, RenderPipeline, ShaderModule};
// use winit::event_loop::EventLoop;
// use winit::{
//     application::ApplicationHandler,
//     event::*,
//     event_loop::ActiveEventLoop,
//     keyboard::{KeyCode, PhysicalKey},
// };

// type Callback = fn();
// type OnKeyCallback = fn(key: KeyCode, key_state: KeyState);
// type GetDrawCallsCallback<'a> = fn() -> Option<Vec<DrawCall<'a>>>;
// type GetCameraStateCallback = fn() -> CameraState;

// pub trait DatasourceWindow {
//     fn get_draw_calls<'a>(&self) -> Vec<DrawCall>;
// }
// pub struct Window<'a> {
//     state: State,
//     on_key: Option<OnKeyCallback>,
//     on_mouse: Option<Callback>,
//     on_quit: Option<Callback>,
//     on_will_render: Option<Callback>,
//     on_did_render: Option<Callback>,
//     on_did_resize: Option<Callback>,
//     // get_draw_calls: GetDrawCallsCallback<'a>,
//     // get_camera_state: GetCameraStateCallback,
//     quit_is_pending: bool,
//     // source: Option<Box<&'a dyn Datasource_window>>,
//     source: Box<&'a dyn DatasourceWindow>,
// }

// // construction
// impl<'a> Window<'_> {
//     // set
//     pub fn set_datasource<b>(&'a mut self, source: &'a dyn DatasourceWindow) {
//         // self.source = Some(Box::new(source));
//     }

//     // create
//     // pub fn new(state: State, get_draw_calls: GetDrawCallsCallback<'a>, get_camera_state: GetCameraStateCallback, source: &'a dyn Datasource_window) -> Window<'a> {
//     pub fn new(state: State, source: &'a dyn DatasourceWindow) -> Window<'a> {
//         Window {
//             state: state,
//             on_key: None,
//             on_mouse: None,
//             on_quit: None,
//             on_will_render: None,
//             on_did_render: None,
//             on_did_resize: None,
//             // get_draw_calls: get_draw_calls,
//             // get_camera_state: get_camera_state,
//             quit_is_pending: false,
//             // source: None,
//             source: Box::new(source),
//         }
//     }
//     pub fn get_state(&self) -> &State {
//         &self.state
//     }
//     pub fn run(&mut self, event_loop: EventLoop<State>) {
//         // self.state = Some(state);
//         // let event_loop = EventLoop::with_user_event().build().unwrap();
//         // event_loop.run_app(self);
//     }
//     pub fn set_on_key_callback(&mut self, cb: OnKeyCallback) {
//         self.on_key = Some(cb);
//     }
//     pub fn set_on_mouse_callback(&mut self, cb: Callback) {
//         self.on_mouse = Some(cb);
//     }
//     pub fn set_on_resize_callback(&mut self, cb: Callback) {
//         self.on_did_resize = Some(cb);
//     }
//     pub fn set_on_will_render_callback(&mut self, cb: Callback) {
//         self.on_did_render = Some(cb);
//     }
//     pub fn set_on_did_render_callback(&mut self, cb: Callback) {
//         self.on_will_render = Some(cb);
//     }
//     pub fn quit(&mut self) {
//         self.quit_is_pending = true;
//     }
// }

// // private methods
// impl Window<'_> {
//     fn handle_key(state: &mut State, event_loop: &ActiveEventLoop, key: KeyCode, pressed: bool) {
//         match (key, pressed) {
//             (KeyCode::Escape, true) => println!("escape down"), // event_loop.exit(),
//             (KeyCode::Escape, false) => println!("escape up"),  //event_loop.exit(),
//             _ => {}
//         }
//     }
//     fn handle_exit(state: &mut State, event_loop: &ActiveEventLoop) {
//         event_loop.exit();
//     }
//     fn handle_resize(state: &mut State, width: u32, height: u32) {
//         if width > 0 && height > 0 {
//             state.config.width = width;
//             state.config.height = height;
//             state.surface.configure(&state.device, &state.config);
//             state.is_surface_configured = true;

//             //Make sure you update the depth_texture after you update config. If you don't, your program will crash as the depth_texture will be a different size than the surface texture.
//             state.depth_texture = super::super::texture::Texture::create_depth_texture(&state.device, &state.config, "depth_texture");
//         }
//     }
//     fn handle_render<'a>(state: &mut State, get: GetDrawCallsCallback<'a>) -> Result<(), wgpu::SurfaceError> {
//         // redraw on window
//         state.window.request_redraw();

//         // We can't render unless the surface is configured
//         if !state.is_surface_configured {
//             return Ok(());
//         }

//         //
//         let output = Window::get_output_texture(state);
//         let mut encoder = Window::get_encoder(state);

//         {
//             let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
//             let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//                 label: Some("Render Pass"),

//                 color_attachments: &[Window::get_color_atatchment(&view)],
//                 depth_stencil_attachment: Window::get_depth_attatchment(state),

//                 occlusion_query_set: None,
//                 timestamp_writes: None,
//             });

//             let draw_calls = get().unwrap();

//             for draw_call in draw_calls {
//                 //  for x in draw_call.mesh {
//                 for i in 0..draw_call.mesh.len() {
//                     let x = draw_call.mesh[i];
//                     let mat = draw_call.materials[i];
//                     let diffuse_bind_group = Window::get_diffuse_binding(&state, mat.textures[0]);
//                     // set render pipeline
//                     let rp = Window::get_render_pipeline(&state, mat.shader.clone());
//                     render_pass.set_pipeline(&rp);

//                     // create the instance buffer
//                     let n_buffer = state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//                         label: Some("Instance Buffer"),
//                         contents: bytemuck::cast_slice(&draw_call.matrix),
//                         usage: wgpu::BufferUsages::VERTEX,
//                     });

//                     // fetch the cached buffers for the mesh
//                     let buffers: (&Buffer, &Buffer) = state.buffer_cache.get_vertex_buffer(&state.device, x);

//                     // set buffers
//                     render_pass.set_vertex_buffer(0, buffers.0.slice(..));
//                     render_pass.set_vertex_buffer(1, n_buffer.slice(..));
//                     render_pass.set_index_buffer(buffers.1.slice(..), wgpu::IndexFormat::Uint32);

//                     // set bind groups
//                     render_pass.set_bind_group(0, &diffuse_bind_group, &[]);
//                     render_pass.set_bind_group(1, &state.camera_bind_group, &[]);

//                     // draw
//                     render_pass.draw_indexed(0..(x.indicies.len() as u32), 0, 0..draw_call.matrix.len() as u32);
//                 }
//             }
//         }

//         // submit commands for execution
//         state.queue.submit(iter::once(encoder.finish()));

//         // present the completed texture
//         output.present();

//         // return a success
//         Ok(())
//     }
//     fn get_diffuse_binding(state: &State, texture: &Texture_asset) -> BindGroup {
//         // let diffuse_bytes = include_bytes!();
//         // let diffuse_bytes = include_bytes!("../../happy-tree.png");
//         // let diffuse_texture = texture::Texture::from_bytes(&state.device, &state.queue, diffuse_bytes, "../happy-tree.png").unwrap(); // CHANGED!

//         let texture_bind_group_layout = state.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
//             entries: &[
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 0,
//                     visibility: wgpu::ShaderStages::FRAGMENT,
//                     ty: wgpu::BindingType::Texture {
//                         multisampled: false,
//                         view_dimension: wgpu::TextureViewDimension::D2,
//                         sample_type: wgpu::TextureSampleType::Float { filterable: true },
//                     },
//                     count: None,
//                 },
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 1,
//                     visibility: wgpu::ShaderStages::FRAGMENT,
//                     // This should match the filterable field of the
//                     // corresponding Texture entry above.
//                     ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
//                     count: None,
//                 },
//             ],
//             label: Some("texture_bind_group_layout"),
//         });

//         let diffuse_bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
//             layout: &texture_bind_group_layout,
//             entries: &[
//                 wgpu::BindGroupEntry {
//                     binding: 0,
//                     resource: wgpu::BindingResource::TextureView(&texture.view), // CHANGED!
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 1,
//                     resource: wgpu::BindingResource::Sampler(&texture.sampler), // CHANGED!
//                 },
//             ],
//             label: Some("diffuse_bind_group"),
//         });

//         diffuse_bind_group
//     }
//     fn get_render_pipeline(state: &State, shader: ShaderModule) -> RenderPipeline {
//         let texture_bind_group_layout = state.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
//             entries: &[
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 0,
//                     visibility: wgpu::ShaderStages::FRAGMENT,
//                     ty: wgpu::BindingType::Texture {
//                         multisampled: false,
//                         view_dimension: wgpu::TextureViewDimension::D2,
//                         sample_type: wgpu::TextureSampleType::Float { filterable: true },
//                     },
//                     count: None,
//                 },
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 1,
//                     visibility: wgpu::ShaderStages::FRAGMENT,
//                     ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
//                     count: None,
//                 },
//             ],
//             label: Some("texture_bind_group_layout"),
//         });
//         let camera_bind_group_layout = state.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
//             entries: &[wgpu::BindGroupLayoutEntry {
//                 binding: 0,
//                 visibility: wgpu::ShaderStages::VERTEX,
//                 ty: wgpu::BindingType::Buffer {
//                     ty: wgpu::BufferBindingType::Uniform,
//                     has_dynamic_offset: false,
//                     min_binding_size: None,
//                 },
//                 count: None,
//             }],
//             label: Some("camera_bind_group_layout"),
//         });
//         let render_pipeline_layout = state.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
//             label: Some("Render Pipeline Layout"),
//             bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
//             push_constant_ranges: &[],
//         });

//         state.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
//             label: Some("Render Pipeline"),
//             layout: Some(&render_pipeline_layout),
//             vertex: wgpu::VertexState {
//                 module: &shader,
//                 entry_point: Some("vs_main"),
//                 // buffers: &[super::model::ModelVertex::desc(), InstanceRaw::desc()],
//                 buffers: &[Vertex::desc(), Matrix4x4::desc()],
//                 compilation_options: Default::default(),
//             },
//             fragment: Some(wgpu::FragmentState {
//                 module: &shader,
//                 entry_point: Some("fs_main"),
//                 targets: &[Some(wgpu::ColorTargetState {
//                     format: state.config.format,
//                     blend: Some(wgpu::BlendState {
//                         color: wgpu::BlendComponent::REPLACE,
//                         alpha: wgpu::BlendComponent::REPLACE,
//                     }),
//                     write_mask: wgpu::ColorWrites::ALL,
//                 })],
//                 compilation_options: Default::default(),
//             }),
//             primitive: wgpu::PrimitiveState {
//                 topology: wgpu::PrimitiveTopology::TriangleList,
//                 strip_index_format: None,
//                 front_face: wgpu::FrontFace::Ccw,
//                 cull_mode: Some(wgpu::Face::Back),
//                 polygon_mode: wgpu::PolygonMode::Fill,
//                 unclipped_depth: false,
//                 conservative: false,
//             },
//             // depth_stencil: Some(wgpu::DepthStencilState {
//             //     format: texture::Texture::DEPTH_FORMAT,
//             //     depth_write_enabled: true,
//             //     depth_compare: wgpu::CompareFunction::Less, // 1.
//             //     stencil: wgpu::StencilState::default(),     // 2.
//             //     bias: wgpu::DepthBiasState::default(),
//             // }),
//             depth_stencil: None,
//             multisample: wgpu::MultisampleState {
//                 count: 1,
//                 mask: !0,
//                 alpha_to_coverage_enabled: false,
//             },
//             // If the pipeline will be used with a multiview render pass, this
//             // indicates how many array layers the attachments will have.
//             multiview: None,
//             // Useful for optimizing shader compilation on Android
//             cache: None,
//         })
//     }

//     fn get_output_texture(state: &State) -> SurfaceTexture {
//         state.surface.get_current_texture().unwrap()
//     }
//     fn get_encoder(state: &State) -> CommandEncoder {
//         state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") })
//     }
//     fn get_color_atatchment<'a>(view: &'a TextureView) -> Option<RenderPassColorAttachment<'a>> {
//         Some(wgpu::RenderPassColorAttachment {
//             view: view,
//             resolve_target: None,
//             ops: wgpu::Operations {
//                 load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 }),
//                 store: wgpu::StoreOp::Store,
//             },
//         })
//     }
//     fn get_depth_attatchment<'a>(state: &'a State) -> Option<RenderPassDepthStencilAttachment<'a>> {
//         Some(wgpu::RenderPassDepthStencilAttachment {
//             view: &state.depth_texture.view,
//             depth_ops: Some(wgpu::Operations {
//                 load: wgpu::LoadOp::Clear(1.0),
//                 store: wgpu::StoreOp::Store,
//             }),
//             stencil_ops: None,
//         })
//     }

//     fn correct_stale_frame(state: &mut State) {
//         let size = state.window.inner_size();
//         Window::handle_resize(state, size.width, size.height);
//     }
// }

// impl ApplicationHandler<State> for Window<'_> {
//     fn resumed(&mut self, event_loop: &ActiveEventLoop) {
//         // #[allow(unused_mut)]
//         // let mut window_attributes = winit::window::Window::default_attributes();

//         // let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
//         // self.state = Some(pollster::block_on(State::new(window)).unwrap());
//     }

//     #[allow(unused_mut)]
//     fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: State) {
//         // self.state = Some(event);
//     }

//     fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: winit::window::WindowId, event: WindowEvent) {
//         // guard - get current state
//         // let state = match &mut self.state {
//         //     Some(canvas) => canvas,
//         //     None => return,
//         // };

//         let state = &mut self.state;

//         // get match for events
//         match event {
//             WindowEvent::CloseRequested => {
//                 Window::handle_exit(state, event_loop);
//                 self.on_quit.unwrap()()
//             }
//             WindowEvent::Resized(size) => Window::handle_resize(state, size.width, size.height),
//             WindowEvent::RedrawRequested => {
//                 // update camera
//                 let camera_state = (self.get_camera_state)();
//                 state.camera_uniform.update_view_proj(&camera_state);
//                 state.queue.write_buffer(&state.camera_buffer, 0, bytemuck::cast_slice(&[state.camera_uniform]));

//                 // try render
//                 match Window::handle_render(state, self.get_draw_calls) {
//                     Err(Lost | Outdated) => Window::correct_stale_frame(state),
//                     Err(e) => panic!(e),
//                     _ => {}
//                 }
//             }
//             WindowEvent::MouseInput { state, button, .. } => match (button, state.is_pressed()) {
//                 (MouseButton::Left, true) => {}
//                 (MouseButton::Left, false) => {}
//                 _ => {}
//             },
//             WindowEvent::KeyboardInput {
//                 event: KeyEvent {
//                     physical_key: PhysicalKey::Code(code),
//                     state: key_state,
//                     ..
//                 },
//                 ..
//             } => {
//                 // state.camera_controller.process_events(&event);
//                 Window::handle_key(state, event_loop, code, key_state.is_pressed());
//                 self.on_key.unwrap()(code, if key_state.is_pressed() { KeyState::Down } else { KeyState::Up });
//             }
//             _ => {}
//         }
//     }

//     fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
//         let _ = (event_loop, cause);
//     }

//     fn device_event(&mut self, event_loop: &ActiveEventLoop, device_id: DeviceId, event: DeviceEvent) {
//         let _ = (event_loop, device_id, event);
//     }

//     fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
//         let _ = event_loop;
//     }

//     fn suspended(&mut self, event_loop: &ActiveEventLoop) {
//         let _ = event_loop;
//     }

//     fn exiting(&mut self, event_loop: &ActiveEventLoop) {
//         let _ = event_loop;
//     }

//     fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {
//         let _ = event_loop;
//     }
// }

// #[derive(PartialEq)]
// pub enum KeyState {
//     Down,
//     Up,
// }
