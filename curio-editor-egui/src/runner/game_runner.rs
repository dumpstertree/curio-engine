//! The game runner thread. **Reverted to a fully private headless wgpu
//! device** (see `capture.rs`'s doc comment for the full reasoning) — this
//! thread now shares no GPU resource with eframe's own rendering, which is
//! what the original Tauri build did too and is what makes this reliable:
//! nothing else in the process ever touches this `Device`/`Queue`/
//! `capture_texture`, so there's no lock contention or ordering dependency
//! with the UI thread's own paint loop to race against.
//!
//! Frames leave this thread as plain RGBA bytes via `capture::push_frame`,
//! picked up by `center_panel.rs` each repaint and uploaded into a
//! persistent `egui::TextureHandle`.
//!
//! Differences from the original Tauri build's `runner2.rs`, everything
//! else preserved:
//! - No `PROJECT`/`SHARED_DATA` `unsafe static`s — project info is passed in
//!   via `Arc<Mutex<Project>>`, and the latest snapshot is published through
//!   `SharedGameData` (a process-wide `Lazy<Mutex<_>>`, since the UI thread
//!   needs to read it from anywhere — but `Mutex` not `unsafe`).
//! - No Tauri `Channel`; frames are published via `capture::push_frame`
//!   instead (a `Mutex<Option<Frame>>` slot — see that module).

use crate::project::Project;
use crate::runner::{
    callbacks::{set_cursor_visible, set_fullscreen, set_resolution},
    capture::{lock_process_memory, record_readback, ReadbackBuffers, CAPTURE_HEIGHT, CAPTURE_WIDTH},
    plugin_loader::{self, load_library},
};

use curio_core::io::asset_cache::AssetCache;
use curio_core::io::asset_database::AssetDatabase;
use curio_core::io::asset_loader::AssetLoader;
use curio_core::{set_services, ComponentState, Curio, CurioCommon, EngineServices, GpuHandle, Logger, PluginGroupState};

use egui_wgpu::wgpu::{Adapter, CommandEncoderDescriptor, Device, DeviceDescriptor, Extent3d, Features, Instance, Limits, PowerPreference, Queue, RequestAdapterOptions, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor};

use libloading::Symbol;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use pollster::FutureExt;
use serde::{Deserialize, Serialize};

use std::{
    path::Path,
    sync::{mpsc::Receiver, Arc},
    time::{Duration, Instant},
};

// ─────────────────────────────────────────────────────────────────────────────
// Shared output snapshot — runner thread writes, UI thread reads
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Default, Clone)]
pub struct SharedGameData {
    // pub forms: FormsSnapshot,
    pub plugin: PluginGroupState,
}

pub static SHARED_DATA: Lazy<Mutex<SharedGameData>> = Lazy::new(|| Mutex::new(SharedGameData::default()));

// ─────────────────────────────────────────────────────────────────────────────
// Input event
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InputEvent {
    Button { code: u32, pressed: bool },
    Axis { code: u32, x: f32, y: f32 },
}

// ─────────────────────────────────────────────────────────────────────────────
// Messages
// ─────────────────────────────────────────────────────────────────────────────

pub enum GameMessage {
    Start,
    Stop,
    Pause,
    Resume,
    Resize(u32, u32),
    Input(InputEvent),
}

// ─────────────────────────────────────────────────────────────────────────────
// Runner state
// ─────────────────────────────────────────────────────────────────────────────

enum RunnerState {
    Stopped,
    Playing,
    Paused,
}

// ─────────────────────────────────────────────────────────────────────────────
// Loaded game plugin
// ─────────────────────────────────────────────────────────────────────────────

pub struct LoadedCurio {
    pub curio: Box<Curio>,
}

pub struct AppInstance {
    pub app_instance: LoadedCurio,
}

impl AppInstance {
    pub fn new(curio: LoadedCurio) -> Self {
        Self { app_instance: curio }
    }

    pub fn forward_input(&mut self, _event: InputEvent) {
        match _event {
            InputEvent::Button { code: _, pressed: _ } => {
                // Wire up to curio_core's input API once it's finalized —
                // left as a no-op in the original Tauri build too.
            }
            InputEvent::Axis { code: _, x: _, y: _ } => {}
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// GameRunner
// ─────────────────────────────────────────────────────────────────────────────

pub struct GameRunner {
    pub logger: Box<Logger>,
    pub assets: Box<AssetLoader>,
    pub device: Option<Arc<Device>>,
    pub queue: Option<Arc<Queue>>,
    pub adapter: Option<Arc<Adapter>>,

    project: Arc<Mutex<Project>>,
    state: RunnerState,
    rx: Receiver<GameMessage>,
    services: Option<Box<EngineServices>>,
    capture_texture: Option<Arc<Texture>>,
    surface_format: TextureFormat,
    loaded_app: Option<AppInstance>,
    // Render thread owns readback exclusively — no shared mutex needed
    readback: Option<ReadbackBuffers>,
}

impl GameRunner {
    pub fn new(rx: Receiver<GameMessage>, project: Arc<Mutex<Project>>) -> Self {
        Self {
            rx,
            project,
            logger: Box::new(Logger::new()),
            assets: Box::new(AssetLoader::new(AssetCache::new(100), AssetDatabase::new())),
            services: None,
            loaded_app: None,
            capture_texture: None,
            readback: None,
            state: RunnerState::Stopped,
            surface_format: TextureFormat::Rgba8UnormSrgb,
            device: None,
            queue: None,
            adapter: None,
        }
    }

    // ── GPU setup ────────────────────────────────────────────────────────────
    // Private headless device — see this module's doc comment for why this
    // is deliberately NOT shared with eframe's own renderer.

    fn setup_gpu(&mut self) {
        let instance = Instance::new(&egui_wgpu::wgpu::InstanceDescriptor {
            backends: egui_wgpu::wgpu::Backends::VULKAN,
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .block_on()
            .expect("No Vulkan adapter found.");

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("curio_headless_device"),
                    required_features: Features::POLYGON_MODE_LINE | Features::BUFFER_BINDING_ARRAY,
                    required_limits: Limits::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .block_on()
            .expect("Failed to create wgpu device");

        let device = Arc::new(device);
        let queue = Arc::new(queue);
        let adapter = Arc::new(adapter);

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
            format: self.surface_format,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC | TextureUsages::COPY_DST,
            view_formats: &[],
        }));

        let readback = ReadbackBuffers::new(device.clone(), CAPTURE_WIDTH, CAPTURE_HEIGHT);

        self.services = Some(Box::new(EngineServices {
            assets: self.assets.as_mut() as *mut AssetLoader,
            logger: self.logger.as_mut() as *mut Logger,
            gpu: GpuHandle {
                device: Arc::as_ptr(&device) as *const (),
                queue: Arc::as_ptr(&queue) as *const (),
                capture_texture: Arc::as_ptr(&capture_texture) as *const (),
                capture_width: CAPTURE_WIDTH,
                capture_height: CAPTURE_HEIGHT,
            },
            // set_fullscreen,
            // set_resolution,
            // set_cursor_visible,
        }));

        let services_ptr = self.services.as_deref().unwrap() as *const EngineServices;
        set_services(services_ptr);

        self.device = Some(device);
        self.queue = Some(queue);
        self.adapter = Some(adapter);
        self.capture_texture = Some(capture_texture);
        self.readback = Some(readback);
    }

    // ── Main loop ────────────────────────────────────────────────────────────

    pub fn run(mut self) {
        lock_process_memory();
        self.setup_gpu();

        let frame_duration = Duration::from_micros(16_667); // ~60fps

        loop {
            let frame_start = Instant::now();

            self.process_messages();

            match self.state {
                RunnerState::Playing => {
                    if let Some(x) = self.loaded_app.as_mut() {
                        x.app_instance.curio.update();
                    }
                    self.render_frame();
                }
                RunnerState::Paused | RunnerState::Stopped => {
                    std::thread::sleep(Duration::from_millis(16));
                    continue;
                }
            }

            let elapsed = frame_start.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }
        }
    }

    // ── Per-frame render ─────────────────────────────────────────────────────

    fn render_frame(&mut self) {
        let (Some(loaded_app), Some(services), Some(capture_texture), Some(readback)) = (self.loaded_app.as_mut(), &self.services, &self.capture_texture, self.readback.as_mut()) else {
            return;
        };

        let device = services.gpu.device();
        let queue = services.gpu.queue();

        // Collect previous frame (blocks until GPU done).
        readback.blocking_collect_and_push(capture_texture.width(), capture_texture.height());

        // Render frame N.
        let output_view = capture_texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: Some("curio_frame_encoder") });

        loaded_app
            .app_instance
            .curio
            .render(capture_texture, &output_view, &mut encoder);

        record_readback(&mut encoder, capture_texture, &readback.write_buf);

        queue.submit(std::iter::once(encoder.finish()));
        readback.kick_map();

        let mut shared_data = SHARED_DATA.lock();
        // shared_data.forms = loaded_app.app_instance.curio.context_snapshot();
        shared_data.plugin = loaded_app.app_instance.curio.get_plugin_group_state();
    }

    // ── Message dispatch ─────────────────────────────────────────────────────

    fn process_messages(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                GameMessage::Start => {
                    let project = self.project.lock().clone();
                    let services_ptr = self.services.as_deref().unwrap() as *const EngineServices;
                    let path = format!("{}/target/release/", project.project_path);
                    let loaded_curio = load_curio(Path::new(&path), services_ptr);
                    let mut app_instance = AppInstance::new(loaded_curio);
                    app_instance.app_instance.curio.window_opened();
                    self.loaded_app = Some(app_instance);
                    self.state = RunnerState::Playing;
                }

                GameMessage::Pause => self.state = RunnerState::Paused,
                GameMessage::Resume => self.state = RunnerState::Playing,

                GameMessage::Stop => {
                    if let Some(app) = self.loaded_app.take() {
                        drop(app);
                    }
                    if let Some(rb) = self.readback.as_mut() {
                        rb.reset();
                    }
                    self.state = RunnerState::Stopped;
                }

                GameMessage::Resize(w, h) => self.resize_capture(w, h),

                GameMessage::Input(event) => {
                    if let Some(app) = self.loaded_app.as_mut() {
                        app.forward_input(event);
                    }
                }
            }
        }
    }

    // ── Resize ───────────────────────────────────────────────────────────────

    fn resize_capture(&mut self, w: u32, h: u32) {
        let Some(device) = &self.device else { return };

        let capture_texture = Arc::new(device.create_texture(&TextureDescriptor {
            label: Some("capture_texture"),
            size: Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: self.surface_format,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC | TextureUsages::COPY_DST,
            view_formats: &[],
        }));

        if let Some(services) = &mut self.services {
            services.gpu.capture_texture = Arc::as_ptr(&capture_texture) as *const ();
            services.gpu.capture_width = w;
            services.gpu.capture_height = h;
        }

        self.readback = Some(ReadbackBuffers::new(device.clone(), w, h));
        self.capture_texture = Some(capture_texture);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Plugin loading
// ─────────────────────────────────────────────────────────────────────────────

type InitCurioFn = unsafe extern "C" fn(gpu: *const EngineServices) -> *mut Curio;
type PeekCurioFn = unsafe extern "C" fn() -> *mut Vec<ComponentState>;

pub fn peek_curio(folder: &Path) -> Box<Vec<ComponentState>> {
    let entries = std::fs::read_dir(folder).expect("plugins folder not found");
    for entry in entries.flatten() {
        let path = entry.path();
        if let Err(e) = load_library(&path) {
            eprintln!("load_library failed for {:?}: {}", path, e);
            continue;
        }
        let lib = plugin_loader::library_slot().lock().unwrap();
        let Some(lib) = lib.as_ref() else { continue };
        let curio = unsafe {
            let f: Symbol<PeekCurioFn> = if let Ok(f) = lib.get(b"curio_peek") { f } else { continue };
            let raw = f();
            if raw.is_null() {
                continue;
            }
            Box::from_raw(raw)
        };
        return curio;
    }
    panic!("No curio plugin found");
}

pub fn load_curio(folder: &Path, gpu: *const EngineServices) -> LoadedCurio {
    let entries = std::fs::read_dir(folder).expect("plugins folder not found");
    for entry in entries.flatten() {
        let path = entry.path();
        let is_plugin = matches!(path.extension().and_then(|e| e.to_str()), Some("so") | Some("dll") | Some("dylib"));
        if !is_plugin {
            continue;
        }
        if let Err(e) = load_library(&path) {
            eprintln!("load_library failed for {:?}: {}", path, e);
            continue;
        }
        let lib = plugin_loader::library_slot().lock().unwrap();
        let Some(lib) = lib.as_ref() else { continue };
        let curio = unsafe {
            let f: Symbol<InitCurioFn> = if let Ok(f) = lib.get(b"curio_init") { f } else { continue };
            let raw = f(gpu);
            if raw.is_null() {
                eprintln!("curio_init null for {:?}", path);
                continue;
            }
            Box::from_raw(raw)
        };
        eprintln!("loaded: {}", curio.meta.name);
        return LoadedCurio { curio };
    }
    panic!("No curio plugin found");
}
