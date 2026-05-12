// src-tauri/src/lib.rs
use serde::Serialize;
use std::path::Path;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use tauri::State;
use winit::platform::wayland::EventLoopBuilderExtWayland;
// use winit::platform::x11::EventLoopBuilderExtX11;
// ─────────────────────────────────────────────────────────────
// Messages the editor sends into the game thread
// ─────────────────────────────────────────────────────────────

enum GameMessage {
    Pause,
    Resume,
    Stop,
}

// ─────────────────────────────────────────────────────────────
// Types matching React's types.ts
// ─────────────────────────────────────────────────────────────

#[derive(Serialize, Clone)]
pub struct ComponentData {
    pub name: String,
    pub fields: serde_json::Value,
}

#[derive(Serialize, Clone)]
pub struct EntityData {
    pub id: u64,
    pub name: String,
    pub children: Vec<EntityData>,
    pub components: Vec<ComponentData>,
}

#[derive(Serialize, Clone)]
pub struct SceneSnapshot {
    pub entities: Vec<EntityData>,
}

// ─────────────────────────────────────────────────────────────
// Editor state
// ─────────────────────────────────────────────────────────────

#[derive(PartialEq)]
enum EditorMode {
    Stopped,
    Playing,
    Paused,
}

struct EditorState {
    mode: EditorMode,
    game_tx: Option<Sender<GameMessage>>,
    game_thread: Option<JoinHandle<()>>,
}

impl EditorState {
    fn new() -> Self {
        Self {
            mode: EditorMode::Stopped,
            game_tx: None,
            game_thread: None,
        }
    }
}

// ─────────────────────────────────────────────────────────────
// The game runner — owns winit + curio, runs on background thread
// ─────────────────────────────────────────────────────────────

use curio_core::{
    collections::{curio_metadata::CurioMetadata, version_number::VersionNumber, window_layout::WindowLayout},
    engine::{
        curio::{load_curio, LoadedCurio},
        curio_common::CurioCommon,
    },
    engine_services::{EngineServices, GpuHandle},
    system_adapters::adapter_system_gpu::SystemGPU,
    Application, AxisCode, ButtonCode, GPUInstance, KeyState, Severity, TextureAsset, Vector3,
};

use egui_wgpu::wgpu::{Adapter, Device, DeviceDescriptor, Features, Instance, Limits, PowerPreference, Queue, RequestAdapterOptions, Surface, SurfaceConfiguration, TextureFormat, TextureUsages};
use pollster::FutureExt;
use std::sync::mpsc::Receiver;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::Window,
};

struct GameRunner {
    rx: Receiver<GameMessage>,
    gpu_instance: Option<Arc<GPUInstance>>,
    gpu: Option<Arc<SystemGPU>>,
    services: Option<Box<EngineServices>>,
    app_instance: Option<LoadedCurio>,
    paused: bool,
    should_stop: bool,
}

impl GameRunner {
    fn new(rx: Receiver<GameMessage>) -> Self {
        Self {
            rx,
            gpu_instance: None,
            gpu: None,
            services: None,
            app_instance: None,
            paused: false,
            should_stop: false,
        }
    }
}
impl GameRunner {
    fn run(mut self) {
        // detect which display server is running
        // and use any_thread() on the appropriate extension
        let event_loop = if std::env::var("WAYLAND_DISPLAY").is_ok() {
            winit::event_loop::EventLoop::builder()
                .with_any_thread(true) // wayland extension
                .build()
                .expect("Failed to build Wayland EventLoop")
        } else {
            panic!("X11 not supported");
            // winit::event_loop::EventLoop::builder()
            //     .with_any_thread(true) // x11 extension
            //     .build()
            //     .expect("Failed to build X11 EventLoop")
        };

        event_loop
            .run_app(&mut self)
            .expect("Failed to run game event loop");

        println!("[game] event loop exited, starting drop...");
        drop(self.app_instance.take()); // drop curio + .so explicitly
        println!("[game] app dropped");
        drop(self.services.take());
        println!("[game] services dropped");
        drop(self.gpu.take());
        println!("[game] gpu dropped");
        drop(self.gpu_instance.take());
        println!("[game] gpu_instance dropped");
        println!("[game] all resources dropped cleanly");
    }
}

// C callbacks the .so needs
#[unsafe(no_mangle)]
pub extern "C" fn set_resolution(_x: i32, _y: i32) {}
#[unsafe(no_mangle)]
pub extern "C" fn set_fullscreen(_x: bool) {}
#[unsafe(no_mangle)]
pub extern "C" fn set_cursor_visible(_x: bool) {}

impl ApplicationHandler for GameRunner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // build window + GPU — same as your existing CabinetWindowOwner::resumed
        let instance = Instance::new(&egui_wgpu::wgpu::InstanceDescriptor {
            backends: egui_wgpu::wgpu::Backends::all(),
            ..Default::default()
        });

        let window = {
            let atts = Window::default_attributes()
                .with_title("Curio")
                .with_inner_size(PhysicalSize::new(1920, 1080))
                .with_resizable(true);
            Arc::new(event_loop.create_window(atts).unwrap())
        };

        let surface = Arc::new(instance.create_surface(window.clone()).unwrap());

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .block_on()
            .expect("No adapter");

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
            .expect("No device");

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let config = Arc::new(SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: 1920,
            height: 1080,
            present_mode: caps.present_modes[0],
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        });

        let depth_texture = Arc::new(create_depth_texture(&device, "DEPTH"));

        self.gpu_instance = Some(Arc::new(GPUInstance {
            device: device.clone(),
            queue: queue.clone(),
            surface: surface.clone(),
            adapter: adapter.clone(),
            window: window.clone(),
            config: config.clone(),
            depth_texture: depth_texture.clone(),
        }));

        let gpu = Arc::new(SystemGPU::new(self.gpu_instance.as_ref().unwrap().clone()));

        self.services = Some(Box::new(EngineServices {
            gpu: GpuHandle {
                device: Arc::as_ptr(&gpu.device) as *const (),
                queue: Arc::as_ptr(&gpu.queue) as *const (),
                config: Arc::as_ptr(&gpu.config) as *const (),
                window: Arc::as_ptr(&gpu.window) as *const (),
                surface: Arc::as_ptr(&gpu.surface) as *const (),
                depth: Arc::as_ptr(&depth_texture) as *const (),
            },
            set_fullscreen: set_fullscreen,
            set_resolution: set_resolution,
            set_cursor_visible: set_cursor_visible,
        }));

        self.gpu = Some(gpu);

        let services_ptr = self.services.as_deref().unwrap() as *const EngineServices;
        let mut loaded = load_curio(Path::new("./plugins"), services_ptr);
        loaded.curio.window_opened();
        self.app_instance = Some(loaded);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                GameMessage::Pause => {
                    println!("[game] received Pause");
                    self.paused = true;
                }
                GameMessage::Resume => {
                    println!("[game] received Resume");
                    self.paused = false;
                }
                GameMessage::Stop => {
                    println!("[game] received Stop");
                    self.should_stop = true;
                }
            }
        }

        if self.should_stop {
            println!("[game] calling window_closed");
            if let Some(app) = &mut self.app_instance {
                app.curio.window_closed();
            }
            println!("[game] calling event_loop.exit()");
            event_loop.exit();
            println!("[game] exit called");
        }
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: winit::window::WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                // check stop here too in case about_to_wait isn't being called
                while let Ok(msg) = self.rx.try_recv() {
                    match msg {
                        GameMessage::Stop => self.should_stop = true,
                        GameMessage::Pause => self.paused = true,
                        GameMessage::Resume => self.paused = false,
                    }
                }

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

                if let Some(gpu) = &self.gpu_instance {
                    gpu.window.request_redraw();
                }
            }
            WindowEvent::CloseRequested => {
                if let Some(app) = &mut self.app_instance {
                    app.curio.window_closed();
                }
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
            WindowEvent::Focused(f) => {
                if let Some(app) = &mut self.app_instance {
                    app.curio.window_focused(f);
                }
            }
            WindowEvent::Occluded(o) => {
                if let Some(app) = &mut self.app_instance {
                    app.curio.window_occluded(o);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(app) = &mut self.app_instance {
                    if let Some(btn) = ButtonCode::from_winit_physical_key(event.physical_key) {
                        let s = if event.state.is_pressed() { KeyState::Down } else { KeyState::Up };
                        app.curio.input_button(btn, s);
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
                    if let Some(btn) = ButtonCode::from_winit_mousebutton(button) {
                        let s = if state.is_pressed() { KeyState::Down } else { KeyState::Up };
                        app.curio.input_button(btn, s);
                    }
                }
            }
            _ => {}
        }
    }
}

// ─────────────────────────────────────────────────────────────
// Tauri commands
// ─────────────────────────────────────────────────────────────

#[tauri::command]
fn press_play(state: State<Mutex<EditorState>>) -> Result<(), String> {
    let mut s = state.lock().unwrap();

    if s.mode == EditorMode::Playing {
        return Ok(());
    }

    // if paused just resume — no new thread needed
    if s.mode == EditorMode::Paused {
        if let Some(tx) = &s.game_tx {
            tx.send(GameMessage::Resume).ok();
        }
        s.mode = EditorMode::Playing;
        return Ok(());
    }

    // create channel — tx stays in EditorState, rx goes into game thread
    let (tx, rx) = mpsc::channel::<GameMessage>();
    s.game_tx = Some(tx);

    s.game_thread = Some(std::thread::spawn(move || {
        GameRunner::new(rx).run();
    }));

    s.mode = EditorMode::Playing;
    Ok(())
}

#[tauri::command]
fn press_stop(state: State<Mutex<EditorState>>) -> Result<(), String> {
    println!("[tauri] press_stop start");

    {
        let mut s = state.lock().unwrap();
        println!("[tauri] press_stop got lock");

        if s.mode == EditorMode::Stopped {
            println!("[tauri] press_stop already stopped");
            return Ok(());
        }

        if let Some(tx) = &s.game_tx {
            tx.send(GameMessage::Stop).ok();
        }

        s.game_tx = None;
        s.game_thread = None;
        s.mode = EditorMode::Stopped;
        println!("[tauri] press_stop updated state");
    }

    println!("[tauri] press_stop end");
    Ok(())
}
#[tauri::command]
fn press_pause(state: State<Mutex<EditorState>>) -> Result<(), String> {
    let mut s = state.lock().unwrap();
    match s.mode {
        EditorMode::Playing => {
            if let Some(tx) = &s.game_tx {
                tx.send(GameMessage::Pause).ok();
            }
            s.mode = EditorMode::Paused;
        }
        EditorMode::Paused => {
            if let Some(tx) = &s.game_tx {
                tx.send(GameMessage::Resume).ok();
            }
            s.mode = EditorMode::Playing;
        }
        EditorMode::Stopped => {}
    }
    Ok(())
}

#[tauri::command]
fn get_scene_snapshot(_state: State<Mutex<EditorState>>) -> SceneSnapshot {
    // TODO: wire to actual world
    SceneSnapshot { entities: vec![] }
}

// ─────────────────────────────────────────────────────────────
// Entry point
// ─────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(EditorState::new()))
        .invoke_handler(tauri::generate_handler![press_play, press_stop, press_pause, get_scene_snapshot,])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ─────────────────────────────────────────────────────────────
// Depth texture helper
// ─────────────────────────────────────────────────────────────

fn create_depth_texture(device: &Device, label: &str) -> TextureAsset {
    let size = egui_wgpu::wgpu::Extent3d { width: 1920, height: 1080, depth_or_array_layers: 1 };

    let texture = device.create_texture(&egui_wgpu::wgpu::TextureDescriptor {
        label: Some(label),
        mip_level_count: 1,
        sample_count: 1,
        dimension: egui_wgpu::wgpu::TextureDimension::D2,
        format: TextureAsset::DEPTH_FORMAT,
        usage: egui_wgpu::wgpu::TextureUsages::RENDER_ATTACHMENT | egui_wgpu::wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
        size,
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
