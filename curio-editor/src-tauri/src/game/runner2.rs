// src-tauri/src/game/runner.rs

use crate::{
    callbacks::{set_cursor_visible, set_fullscreen, set_resolution},
    game::capture::{capture_frame, CAPTURE_HEIGHT, CAPTURE_WIDTH},
    PROJECT, SHARED_DATA,
};

use curio_core::{
    engine_services::{EngineServices, GpuHandle},
    load_curio, AxisCode, ButtonCode, ButtonPressed, CurioCommon, FieldState, FormsSnapshot, GPUInstance, LedgerSnapshot, LoadedCurio, ObjectState, TabGroupState, TextureAsset, Vector3,
};

use egui_wgpu::wgpu::{
    Adapter, Buffer, BufferDescriptor, BufferUsages, Device, DeviceDescriptor, Extent3d, Features, Instance, Limits, PowerPreference, Queue, RequestAdapterOptions, Surface, SurfaceCapabilities, SurfaceConfiguration, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

use pollster::FutureExt;
use serde::Serialize;

use std::{
    cell::RefCell,
    default,
    path::Path,
    sync::{mpsc::Receiver, Arc, Mutex},
    vec,
};

use tauri::AppHandle;

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    platform::x11::EventLoopBuilderExtX11,
    window::Window,
};

pub enum State {
    Stopped,
    Playing,
    Paused,
}
pub struct AppInstance {
    // the actual curio
    app_instance: LoadedCurio,

    // frame data
    frame_counter: u32,
}
impl AppInstance {
    pub fn new(curio: LoadedCurio) -> AppInstance {
        AppInstance { app_instance: curio, frame_counter: 0 }
    }
    pub fn pause(&self) {}
    pub fn stop(&self) {}
    pub fn resume(&self) {}
    pub fn resize(&self) {}
}
impl ApplicationHandler for AppInstance {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {}

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {}

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: winit::window::WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                // refresh the curio
                self.app_instance.curio.application_refresh();

                // update the frame
                self.frame_counter = self.frame_counter.wrapping_add(1);

                // update the state info
                if let Ok(mut shared_data) = SHARED_DATA.lock() {
                    shared_data.forms = self.app_instance.curio.context_snapshot();
                    // shared_data.ledger = self.app_instance.curio.ledger_snapshot();
                    shared_data.plugin = self.app_instance.curio.tab_snapshot();
                }
            }

            WindowEvent::CloseRequested => {}

            WindowEvent::Resized(_) => {
                // self.app_instance.window_resized();
            }

            WindowEvent::Moved(_) => {
                self.app_instance.curio.window_moved();
            }

            WindowEvent::Focused(focused) => {
                self.app_instance.curio.window_focused(focused);
            }

            WindowEvent::Occluded(occluded) => {
                self.app_instance.curio.window_occluded(occluded);
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(button) = ButtonCode::from_winit_physical_key(event.physical_key) {
                    let state = if event.state.is_pressed() { ButtonPressed::Down } else { ButtonPressed::Up };

                    self.app_instance.curio.input_button(button, state);
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.app_instance
                    .curio
                    .input_axis(AxisCode::Cursor, Vector3::new(position.x as f32, position.y as f32, 0.0));
            }

            WindowEvent::MouseInput { state, button, .. } => {
                if let Some(button) = ButtonCode::from_winit_mousebutton(button) {
                    let state = if state.is_pressed() { ButtonPressed::Down } else { ButtonPressed::Up };

                    self.app_instance.curio.input_button(button, state);
                }
            }

            _ => {}
        }
    }
}
pub enum GameMessage2 {
    Start,
    Stop,
    Pause,
    Resume,
    Resize(u32, u32),
}

pub struct GameRunner2<'a> {
    state: State,
    app_handle: AppHandle,
    rx: Receiver<GameMessage2>,

    gpu_instance: Option<Arc<GPUInstance>>,
    services: Option<Box<EngineServices>>,

    capture_texture: Option<Arc<Texture>>,
    readback_buffer: Option<Buffer>,
    loaded_app: Option<AppInstance>,
    surface_format: TextureFormat,
    surface: Option<Arc<Surface<'a>>>,
}

impl GameRunner2<'_> {
    pub fn new(rx: Receiver<GameMessage2>, app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            rx,
            gpu_instance: None,
            services: None,
            loaded_app: None,
            capture_texture: None,
            readback_buffer: None,
            state: State::Stopped,
            surface_format: TextureFormat::Rgba8UnormSrgb,
            surface: None,
        }
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::builder()
            .with_x11()
            .with_any_thread(true)
            .build()
            .expect("failed to build event loop");

        event_loop.run_app(&mut self).expect("failed to run app");
    }

    fn process_messages(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                GameMessage2::Start => {
                    unsafe {
                        let Some(ref project) = PROJECT else {
                            panic!("No Loaded Project");
                        };

                        let Ok(guard) = project.lock() else {
                            panic!("No Loaded Project");
                        };

                        // convert the saved services to a pointer
                        let services_ptr = self.services.as_deref().unwrap() as *const EngineServices;

                        // get build path from project
                        let path = format!("{}{}", guard.project_path, "/target/release/");

                        // load the curio from project directory
                        let loaded_curio = load_curio(Path::new(&path), services_ptr);

                        // create an app instance
                        let app_instance = AppInstance::new(loaded_curio);

                        // save the app
                        self.loaded_app = Some(app_instance);

                        // open the new window
                        if let Some(loaded_app) = self.loaded_app.as_mut() {
                            loaded_app.app_instance.curio.window_opened();
                        }
                    }
                    // set the state as now playing
                    self.state = State::Playing;
                }
                GameMessage2::Pause => {
                    // set as paused
                    self.state = State::Paused;
                }
                GameMessage2::Resume => {
                    // set as resumed
                    self.state = State::Playing;
                }
                GameMessage2::Stop => {
                    // teardown the app
                    if let Some(app) = self.loaded_app.take() {
                        app.stop();
                        drop(app);
                        println!("dropped from stop");
                    }

                    // update state
                    self.state = State::Stopped;
                }
                GameMessage2::Resize(w, h) => {}
            }
        }
    }
}

impl GameRunner2<'_> {
    fn get_device_queue(&self, adapter: Arc<Adapter>) -> (Arc<Device>, Arc<Queue>) {
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    required_features: Features::POLYGON_MODE_LINE | Features::BUFFER_BINDING_ARRAY,
                    required_limits: Limits::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .block_on()
            .expect("no device");

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        (device, queue)
    }
    fn get_adapter(&self, instance: &Instance, surface: Arc<Surface>) -> Arc<Adapter> {
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .block_on()
            .expect("no adapter");

        let adapter = Arc::new(adapter);
        adapter
    }
    fn get_surface(&self, instance: &Instance, window: Arc<Window>) -> Arc<Surface<'static>> {
        Arc::new(instance.create_surface(window.clone()).unwrap())
    }
    fn get_window(&self, event_loop: &ActiveEventLoop) -> Arc<Window> {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Curio")
                        .with_inner_size(PhysicalSize::new(1, 1))
                        .with_visible(false)
                        .with_resizable(true),
                )
                .unwrap(),
        );
        window
    }
    fn get_config(&self, format: TextureFormat, width: u32, height: u32, capabilities: &SurfaceCapabilities) -> Arc<SurfaceConfiguration> {
        let config = Arc::new(SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
            format,
            width,
            height,
            present_mode: capabilities.present_modes[0],
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        });

        config
    }
    fn get_format(&self, capabilities: &SurfaceCapabilities) -> TextureFormat {
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(capabilities.formats[0]);

        format
    }
}
impl ApplicationHandler for GameRunner2<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // create const values
        let width = 1920;
        let height = 720;

        //
        let instance = Instance::new(&egui_wgpu::wgpu::InstanceDescriptor {
            backends: egui_wgpu::wgpu::Backends::all(),
            ..Default::default()
        });

        // generate everything needed for rendering
        let window = self.get_window(event_loop);
        let surface = self.get_surface(&instance, window.clone());
        let adapter = self.get_adapter(&instance, surface.clone());
        let device_queue = self.get_device_queue(adapter.clone());
        let capabilities = surface.get_capabilities(&adapter.clone());
        let device = device_queue.0;
        let queue = device_queue.1;
        let format = self.get_format(&capabilities);
        let config = self.get_config(format, width, height, &capabilities);
        let depth_texture = Arc::new(create_depth_texture(&device, width, height));

        // configure the sureface for rendering - make sure to call this again when changing resolution
        surface.configure(&device, &config);

        //
        let capture_texture = Arc::new(device.create_texture(&TextureDescriptor {
            label: Some("capture_texture"),

            size: Extent3d {
                width: CAPTURE_WIDTH,
                height: CAPTURE_HEIGHT,
                depth_or_array_layers: 1,
            },

            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC | TextureUsages::COPY_DST,
            view_formats: &[],
        }));

        let bytes_per_row = crate::utils::align_to(CAPTURE_WIDTH * 4, 256);

        //
        let readback_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("readback_buffer"),
            size: (bytes_per_row * CAPTURE_HEIGHT) as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // save values
        self.capture_texture = Some(capture_texture.clone());
        self.readback_buffer = Some(readback_buffer);
        self.surface = Some(surface.clone());
        self.surface_format = format;

        // save gpu
        self.gpu_instance = Some(Arc::new(GPUInstance {
            device: device.clone(),
            queue: queue.clone(),
            surface: surface.clone(),
            adapter: adapter.clone(),
            window: window.clone(),
            config: config.clone(),
            depth_texture: depth_texture.clone(),
        }));

        // get the gpu or panic
        let Some(ref gpu) = self.gpu_instance else {
            panic!();
        };

        // save services
        self.services = Some(Box::new(EngineServices {
            gpu: GpuHandle {
                device: Arc::as_ptr(&gpu.device) as *const (),
                queue: Arc::as_ptr(&gpu.queue) as *const (),
                config: Arc::as_ptr(&gpu.config) as *const (),
                window: Arc::as_ptr(&gpu.window) as *const (),
                surface: Arc::as_ptr(&gpu.surface) as *const (),
                depth: Arc::as_ptr(&depth_texture) as *const (),
                capture_texture: Arc::as_ptr(&capture_texture) as *const (),
                capture_width: CAPTURE_WIDTH,
                capture_height: CAPTURE_HEIGHT,
            },

            set_fullscreen,
            set_resolution,
            set_cursor_visible,
        }));

        // draw first frame
        if let Some(gpu) = &self.gpu_instance {
            gpu.window.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: winit::window::WindowId, event: WindowEvent) {
        // proccess any recieved messages
        self.process_messages();

        //
        if let Some(loaded_app) = &self.loaded_app {
            // to reduce overhead we only render every other frame currently
            // if loaded_app.frame_counter % 2 == 0 {
            // make sure all the needed data is set
            if let (Some(gpu), Some(texture), Some(buffer)) = (&self.gpu_instance, &self.capture_texture, &self.readback_buffer) {
                // recapture the frame
                capture_frame(self.app_handle.clone(), &gpu.device, &gpu.queue, texture.clone(), buffer, self.surface_format);
            }
            // }
        }

        //
        if let Some(gpu) = &self.gpu_instance {
            gpu.window.request_redraw();
        }

        // pass on events
        match self.state {
            State::Playing => {
                if let Some(app) = self.loaded_app.as_mut() {
                    app.window_event(event_loop, window_id, event);
                }
            }
            _ => {}
        }
    }
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // proccess any recieved messages
        self.process_messages();
    }
}

fn create_depth_texture(device: &Device, width: u32, height: u32) -> TextureAsset {
    let size = Extent3d { width: width, height: height, depth_or_array_layers: 1 };

    let texture = device.create_texture(&TextureDescriptor {
        label: Some("depth_texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureAsset::DEPTH_FORMAT,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    let view = texture.create_view(&egui_wgpu::wgpu::TextureViewDescriptor::default());

    let sampler = device.create_sampler(&egui_wgpu::wgpu::SamplerDescriptor {
        address_mode_u: egui_wgpu::wgpu::AddressMode::ClampToEdge,
        address_mode_v: egui_wgpu::wgpu::AddressMode::ClampToEdge,
        address_mode_w: egui_wgpu::wgpu::AddressMode::ClampToEdge,
        mag_filter: egui_wgpu::wgpu::FilterMode::Linear,
        min_filter: egui_wgpu::wgpu::FilterMode::Linear,
        mipmap_filter: egui_wgpu::wgpu::FilterMode::Nearest,
        compare: Some(egui_wgpu::wgpu::CompareFunction::LessEqual),
        lod_min_clamp: 0.0,
        lod_max_clamp: 100.0,
        ..Default::default()
    });

    TextureAsset { texture, view, sampler }
}
