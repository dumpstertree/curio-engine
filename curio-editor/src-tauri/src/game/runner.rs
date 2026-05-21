// src-tauri/src/game/runner.rs

use crate::{
    callbacks::{set_cursor_visible, set_fullscreen, set_resolution},
    game::capture::{capture_frame, CAPTURE_HEIGHT, CAPTURE_WIDTH},
    SHARED_DATA,
};

use curio_core::{
    engine_services::{EngineServices, GpuHandle},
    load_curio, AxisCode, ButtonCode, ButtonPressed, CurioCommon, FieldState, FormsSnapshot, GPUInstance, LedgerSnapshot, LoadedCurio, ObjectState, TabGroupState, TextureAsset, Vector3,
};

use egui_wgpu::wgpu::{Buffer, BufferDescriptor, BufferUsages, Device, DeviceDescriptor, Extent3d, Features, Instance, Limits, PowerPreference, Queue, RequestAdapterOptions, Surface, SurfaceConfiguration, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};

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
    platform::wayland::EventLoopBuilderExtWayland,
    window::Window,
};

pub enum GameMessage {
    Pause,
    Resume,
    Stop,
    Resize(u32, u32),
}

pub struct GameRunner<'a> {
    app_handle: AppHandle,
    rx: Receiver<GameMessage>,

    gpu_instance: Option<Arc<GPUInstance>>,
    // gpu: Option<Arc<SystemGPU>>,
    services: Option<Box<EngineServices>>,
    app_instance: Option<LoadedCurio>,

    paused: bool,
    should_stop: bool,

    capture_texture: Option<Arc<Texture>>,
    readback_buffer: Option<Buffer>,

    surface_format: TextureFormat,
    surface: Option<Arc<Surface<'a>>>,

    frame_counter: u32,
}

impl GameRunner<'_> {
    pub fn new(rx: Receiver<GameMessage>, app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            rx,

            gpu_instance: None,
            // gpu: None,
            services: None,
            app_instance: None,

            paused: false,
            should_stop: false,

            capture_texture: None,
            readback_buffer: None,

            surface_format: TextureFormat::Rgba8UnormSrgb,

            frame_counter: 0,
            surface: None,
        }
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::builder()
            .with_any_thread(true)
            .build()
            .expect("failed to build event loop");

        event_loop.run_app(&mut self).expect("failed to run app");

        drop(self.app_instance.take());
        drop(self.services.take());
        // drop(self.gpu.take());
        drop(self.gpu_instance.take());
    }

    fn process_messages(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                GameMessage::Pause => {
                    self.paused = true;
                }

                GameMessage::Resume => {
                    self.paused = false;
                }

                GameMessage::Stop => {
                    self.should_stop = true;
                }
                GameMessage::Resize(w, h) => {
                    if let Some(x) = self.services.as_mut() {
                        x.set_resolution2(w, h);
                    }
                }
            }
        }
    }
}

impl ApplicationHandler for GameRunner<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let instance = Instance::new(&egui_wgpu::wgpu::InstanceDescriptor {
            backends: egui_wgpu::wgpu::Backends::all(),
            ..Default::default()
        });

        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Curio")
                        .with_inner_size(PhysicalSize::new(1, 1))
                        .with_resizable(true),
                )
                .unwrap(),
        );

        let surface = Arc::new(instance.create_surface(window.clone()).unwrap());

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .block_on()
            .expect("no adapter");

        let adapter = Arc::new(adapter);

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

        let caps = surface.get_capabilities(&adapter);

        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        self.surface_format = format;

        let width = 1920;
        let height = 720;
        let config = Arc::new(SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,

            format,

            width: width,
            height: height,

            present_mode: caps.present_modes[0],
            alpha_mode: caps.alpha_modes[0],

            view_formats: vec![],

            desired_maximum_frame_latency: 2,
        });

        let depth_texture = Arc::new(create_depth_texture(&device, width, height));

        surface.configure(&device, &config);

        self.gpu_instance = Some(Arc::new(GPUInstance {
            device: device.clone(),
            queue: queue.clone(),
            surface: surface.clone(),
            adapter: adapter.clone(),
            window: window.clone(),
            config: config.clone(),
            depth_texture: depth_texture.clone(),
        }));

        // let gpu = Arc::new(SystemGPU::new(self.gpu_instance.as_ref().unwrap().clone()));

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

        let readback_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("readback_buffer"),

            size: (bytes_per_row * CAPTURE_HEIGHT) as u64,

            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,

            mapped_at_creation: false,
        });

        self.capture_texture = Some(capture_texture.clone());

        self.readback_buffer = Some(readback_buffer);

        let Some(ref gpu) = self.gpu_instance else {
            panic!();
        };

        self.surface = Some(surface.clone());

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

        // self.gpu = Some(gpu);

        let services_ptr = self.services.as_deref().unwrap() as *const EngineServices;

        let mut loaded = load_curio(Path::new("/home/dumpstertree/Git/Rust/system_test/target/release/"), services_ptr);

        loaded.curio.window_opened();

        self.app_instance = Some(loaded);

        if let Some(gpu) = &self.gpu_instance {
            gpu.window.request_redraw();
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.process_messages();

        if self.should_stop {
            if let Some(app) = &mut self.app_instance {
                app.curio.window_closed();
            }

            event_loop.exit();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: winit::window::WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                self.process_messages();

                if self.should_stop {
                    if let Some(app) = &mut self.app_instance {
                        app.curio.window_closed();
                    }

                    event_loop.exit();

                    return;
                }

                if !self.paused {
                    if let Some(app) = &mut self.app_instance {
                        app.curio.application_refresh();
                    }
                }

                self.frame_counter = self.frame_counter.wrapping_add(1);

                if self.frame_counter % 2 == 0 {
                    if let (Some(gpu), Some(texture), Some(buffer)) = (&self.gpu_instance, &self.capture_texture, &self.readback_buffer) {
                        capture_frame(self.app_handle.clone(), &gpu.device, &gpu.queue, texture.clone(), buffer, self.surface_format);
                    }
                }

                if let Some(gpu) = &self.gpu_instance {
                    gpu.window.request_redraw();
                }

                if let Ok(mut shared_data) = SHARED_DATA.lock() {
                    if let Some(app_instance) = &self.app_instance {
                        shared_data.forms = app_instance.curio.context_snapshot();
                        shared_data.ledger = app_instance.curio.ledger_snapshot();
                        shared_data.plugin = app_instance.curio.tab_snapshot();
                    }
                }
            }

            WindowEvent::CloseRequested => {
                self.should_stop = true;
            }

            WindowEvent::Resized(_) => {
                if let Some(app) = &mut self.app_instance {
                    app.curio.window_resized();
                }
            }

            WindowEvent::Moved(_) => {
                if let Some(app) = &mut self.app_instance {
                    app.curio.window_moved();
                }
            }

            WindowEvent::Focused(focused) => {
                if let Some(app) = &mut self.app_instance {
                    app.curio.window_focused(focused);
                }
            }

            WindowEvent::Occluded(occluded) => {
                if let Some(app) = &mut self.app_instance {
                    app.curio.window_occluded(occluded);
                }
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(app) = &mut self.app_instance {
                    if let Some(button) = ButtonCode::from_winit_physical_key(event.physical_key) {
                        let state = if event.state.is_pressed() { ButtonPressed::Down } else { ButtonPressed::Up };

                        app.curio.input_button(button, state);
                    }
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                if let Some(app) = &mut self.app_instance {
                    app.curio
                        .input_axis(AxisCode::Cursor, Vector3::new(position.x as f32, position.y as f32, 0.0));
                }
            }

            WindowEvent::MouseInput { state, button, .. } => {
                if let Some(app) = &mut self.app_instance {
                    if let Some(button) = ButtonCode::from_winit_mousebutton(button) {
                        let state = if state.is_pressed() { ButtonPressed::Down } else { ButtonPressed::Up };

                        app.curio.input_button(button, state);
                    }
                }
            }

            _ => {}
        }
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
