//! Standalone Curio runner — opens a window and presents a compiled
//! game plugin (`curio_init` export) directly to the screen, with no
//! editor UI in front of it.
//!
//! Where the egui editor runs the same game on a private headless device
//! and copies frames back to CPU (`curio_editor/src/runner/game_runner.rs`
//! + `capture.rs`), this binary draws the game straight onto the swapchain
//! and presents it through winit. This is the rendering contract the game
//! plugins expect: `system_component_default/rendering` renders to
//! whatever output view it is handed and only copies into
//! `services.gpu.capture_texture` when one is set — we deliberately leave
//! it null here, so the game's capture step is skipped.
//!
//! GPU setup follows the editor's proven `game_runner.rs` setup (same
//! pinned wgpu version, same required features) so the raw-pointer casts
//! inside the plugin's `Gpu` struct resolve to identical wgpu types.

use crate::plugin_loader::{self, load_library, lock_process_memory};
use crate::project::Project;

use libloading::Symbol;
use pollster::FutureExt;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::Window,
};

use curio_core::{
    AssetCache,
    AssetDatabase,
    AssetPipeline,
    AxisCode,
    ButtonCode,
    ButtonPressed,
    Curio,
    CurioCommon,
    Gpu,
    Logger,
    Portal,
    Services,
    Severity,
    Vector3,
};

use egui_wgpu::wgpu::{
    Adapter,
    CommandEncoderDescriptor,
    Device,
    DeviceDescriptor,
    Features,
    Instance,
    Limits,
    PowerPreference,
    Queue,
    RequestAdapterOptions,
    Surface,
    SurfaceConfiguration,
    TextureFormat,
    TextureViewDescriptor,
    TextureUsages,
};

use std::{path::Path, ptr, sync::Arc};

pub struct CurioCabinet {}
impl CurioCabinet {
    pub fn put_on_display(project: Project) {
        Curio::log(Severity::Info, "Putting Curio on display...");

        let mut window_owner = CabinetWindowOwner::new(Portal::fullscreen_1080(), project);

        window_owner.run();
    }
}

pub struct CabinetWindowOwner {
    _did_run: bool,
    // Boxed so the Services the plugin sees has a stable heap address
    services: Option<Box<Services>>,
    // keeps the LoadedCurio (and its open .so) alive
    app_instance: Option<LoadedCurio>,
    _portal: Portal,
    project: Project,
    // both must outlive the Services pointers into them
    logger: Box<Logger>,
    assets: Box<AssetPipeline>,
    window: Option<Arc<Window>>,
    surface: Option<Arc<Surface<'static>>>,
    adapter: Option<Arc<Adapter>>,
    device: Option<Arc<Device>>,
    queue: Option<Arc<Queue>>,
    surface_config: Option<Arc<SurfaceConfiguration>>,
}
impl CabinetWindowOwner {
    pub fn new(portal: Portal, project: Project) -> CabinetWindowOwner {
        CabinetWindowOwner {
            _did_run: false,
            services: None,
            app_instance: None,
            _portal: portal,
            project,
            logger: Box::new(Logger::new(Severity::Info)),
            assets: Box::new(AssetPipeline::new(AssetCache::new(100), AssetDatabase::new())),
            window: None,
            surface: None,
            adapter: None,
            device: None,
            queue: None,
            surface_config: None,
        }
    }
}
impl CabinetWindowOwner {
    pub fn run(&mut self) {
        // guard - dont run if already running
        if self._did_run {
            return;
        }

        // enable flag
        self._did_run = true;

        // the copied plugin mapping must stay pinned in RAM while it runs
        lock_process_memory();

        // build an event loop to base everything off of
        let Ok(event_loop) = EventLoop::builder().build() else {
            panic!("Failed to build an EventLoop parent for Curio");
        };

        // run the app. this will continue to loop
        let Ok(_result) = event_loop.run_app(self) else {
            panic!("Failed to run Curio");
        };
    }

    // ── Frame render ─────────────────────────────────────────────────────────
    // The game renders into the swapchain texture we hand it; nothing is
    // copied back to CPU — what you see in this window IS the application.

    fn render_frame(&mut self) {
        let Some(app_instance) = &mut self.app_instance else {
            return;
        };
        let Some(surface) = &self.surface else {
            return;
        };
        let Some(device) = &self.device else {
            return;
        };
        let Some(queue) = &self.queue else {
            return;
        };

        app_instance.curio.update();

        let Ok(mut current) = surface.get_current_texture() else {
            // swapchain lost/minimized — skip this frame
            return;
        };

        let output_view = current.texture.create_view(&egui_wgpu::wgpu::TextureViewDescriptor::default());
        let mut encoder = device.create_command_encoder(&egui_wgpu::wgpu::CommandEncoderDescriptor { label: Some("curio_frame_encoder") });

        app_instance
            .curio
            .render(&current.texture, &output_view, &mut encoder);

        queue.submit(std::iter::once(encoder.finish()));
        current.present();
    }

    // ── Resize ───────────────────────────────────────────────────────────────
    // The game reads services.gpu.capture_width/height for its viewport
    // math, so those must follow the window.

    pub fn resize_surface(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        // ignore minimizing/hide events
        if size.width == 0 || size.height == 0 {
            return;
        }
        let Some(surface) = &self.surface else {
            return;
        };
        let Some(device) = &self.device else {
            return;
        };

        if let Some(config) = &self.surface_config {
            let mut resized = (**config).clone();
            if resized.width == size.width && resized.height == size.height {
                return;
            }
            resized.width = size.width;
            resized.height = size.height;

            surface.configure(&*device, &resized);
            self.surface_config = Some(Arc::new(resized));
        }

        if let Some(services) = &mut self.services {
            services.gpu.capture_width = size.width;
            services.gpu.capture_height = size.height;
        }
    }
}

impl ApplicationHandler for CabinetWindowOwner {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        // ── window ───────────────────────────────────────────────────────────
        let atts = Window::default_attributes()
            .with_title("Curio")
            .with_inner_size(PhysicalSize::new(self._portal.width, self._portal.height))
            .with_resizable(true);

        let Ok(window) = event_loop.create_window(atts) else {
            panic!("Failed to create Window");
        };
        let window = Arc::new(window);

        // ── gpu stack (same shape as the editor's game_runner setup) ─────────
        let instance = Instance::new(&egui_wgpu::wgpu::InstanceDescriptor {
            backends: egui_wgpu::wgpu::Backends::all(),
            ..Default::default()
        });

        let Ok(surface) = instance.create_surface(window.clone()) else {
            panic!("Failed to create Surface for Window");
        };

        let Some(adapter) = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .block_on()
        else {
            panic!("Failed to create Adapter");
        };

        let Ok(device_queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("curio_runner_device"),
                    required_features: Features::POLYGON_MODE_LINE | Features::BUFFER_BINDING_ARRAY,
                    required_limits: Limits::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .block_on()
        else {
            panic!("Failed to create Device or Queue");
        };
        let device = Arc::new(device_queue.0);
        let queue = Arc::new(device_queue.1);
        let adapter = Arc::new(adapter);

        // ── swapchain config ─────────────────────────────────────────────────
        let surface_capabilities = surface.get_capabilities(&adapter);

        let surface_format: TextureFormat = surface_capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_capabilities.formats[0]);

        let window_size = window.inner_size();
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&*device, &config);

        // ── services ─────────────────────────────────────────────────────────
        // services is Boxed so it has a stable heap address
        // the Box lives on self — won't drop until CabinetWindowOwner drops
        self.services = Some(Box::new(Services {
            assets: self.assets.as_mut() as *mut AssetPipeline,
            logger: self.logger.as_mut() as *mut Logger,
            gpu: Gpu {
                device: Arc::as_ptr(&device) as *const (),
                queue: Arc::as_ptr(&queue) as *const (),
                // no capture texture — the game draws straight to the swapchain
                capture_texture: ptr::null(),
                capture_width: config.width,
                capture_height: config.height,
            },
        }));

        // get a raw pointer to the Boxed services — stable as long as the Box
        // lives on self
        let services_ptr: *const Services = self.services.as_deref().unwrap();

        // the one static — the plugin's curio_init resolves it
        Services::set(services_ptr);

        // ── load the application plugin and open ─────────────────────────────
        let path = format!("{}/target/release/", self.project.project_path);
        let stem = expected_plugin_stem(self.project.project_path.as_str());
        let mut loaded = load_curio(Path::new(&path), services_ptr, stem.as_deref());

        loaded.curio.window_opened();

        // keep LoadedCurio on self so the .so stays loaded
        self.app_instance = Some(loaded);

        self.window = Some(window);
        self.surface = Some(Arc::new(surface));
        self.adapter = Some(adapter);
        self.device = Some(device);
        self.queue = Some(queue);
        self.surface_config = Some(Arc::new(config));

        // kick off the redraw loop — the first frame renders the application
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn window_event(&mut self, _: &winit::event_loop::ActiveEventLoop, _: winit::window::WindowId, event: winit::event::WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                self.render_frame();

                // keep the loop going — this window IS the application
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(app_instance) = &mut self.app_instance {
                    app_instance.curio.window_resized();
                }
                self.resize_surface(physical_size);
            }
            WindowEvent::Moved(_physical_position) => {
                if let Some(app_instance) = &mut self.app_instance {
                    app_instance.curio.window_moved();
                }
            }
            WindowEvent::Focused(is_focused) => {
                if let Some(app_instance) = &mut self.app_instance {
                    app_instance.curio.window_focused(is_focused);
                }
            }
            WindowEvent::Occluded(is_occluded) => {
                if let Some(app_instance) = &mut self.app_instance {
                    app_instance.curio.window_occluded(is_occluded);
                }
            }
            WindowEvent::KeyboardInput { device_id: _, event, is_synthetic: _ } => {
                if let Some(app_instance) = &mut self.app_instance {
                    if let Some(button_code) = ButtonCode::from_winit_physical_key(event.physical_key) {
                        // get the current button state to propagate
                        let state = if event.state.is_pressed() { ButtonPressed::Down } else { ButtonPressed::Up };

                        // if button was resolved we propagate the event
                        app_instance.curio.input_button(button_code, state);
                    }
                }
            }
            WindowEvent::CursorMoved { device_id: _, position } => {
                if let Some(app_instance) = &mut self.app_instance {
                    app_instance
                        .curio
                        .input_axis(AxisCode::Cursor, Vector3::new(position.x as f32, position.y as f32, 0.0));
                }
            }
            WindowEvent::MouseWheel { device_id: _, delta: _, phase: _ } => {}
            WindowEvent::MouseInput { device_id: _, state, button } => {
                if let Some(app_instance) = &mut self.app_instance {
                    // if button was resolved we propagate the event
                    if let Some(button_code) = ButtonCode::from_winit_mousebutton(button) {
                        // get the current button state to propagate
                        let state = if state.is_pressed() { ButtonPressed::Down } else { ButtonPressed::Up };

                        // if button was resolved we propagate the event
                        app_instance.curio.input_button(button_code, state);
                    }
                }
            }

            _ => {}
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Plugin loading — same contract as the editor's runner: the project's own
// crate is tried first ('first .so with a curio_init wins' can never boot
// somebody else's game), and a plugin that panics in curio_init must not
// take the host down with it.
// ─────────────────────────────────────────────────────────────────────────────

type InitCurioFn = unsafe extern "C" fn(gpu: *const Services) -> *mut Curio;

/// Derive the plugin file stem the project's own crate produces
/// (`package.name` with dashes as underscores, cargo's `lib` prefix).
pub fn expected_plugin_stem(project_path: &str) -> Option<String> {
    let manifest = std::fs::read_to_string(format!("{project_path}/Cargo.toml")).ok()?;
    let mut in_package = false;
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_package = line == "[package]";
            continue;
        }
        if in_package && line.starts_with("name") {
            let name = line.split('=').nth(1)?.trim().trim_matches(['"', '\'']);
            return Some(format!("lib{}", name.replace('-', "_")));
        }
    }
    None
}

pub struct LoadedCurio {
    pub curio: Box<Curio>,
}

pub fn load_curio(folder: &Path, gpu: *const Services, preferred_stem: Option<&str>) -> LoadedCurio {
    println!("folder: {}", folder.as_os_str().to_str().unwrap());
    let entries = std::fs::read_dir(folder).unwrap_or_else(|e| panic!("plugins folder not found at {:?}: {e}", folder));
    let mut plugins: Vec<_> = entries.flatten().map(|path| path.path()).filter(|path| matches!(path.extension().and_then(|e| e.to_str()), Some("so") | Some("dll") | Some("dylib"))).collect();
    // A build dir may hold plugins from other projects (shared targets,
    // leftovers). Always try this project's own crate first.
    if let Some(stem) = preferred_stem {
        plugins.sort_by_key(|path| if path.file_stem().and_then(|s| s.to_str()) == Some(stem) { 0 } else { 1 });
    }
    for path in plugins {
        if let Err(e) = load_library(&path) {
            eprintln!("load_library failed for {:?}: {}", path, e);
            continue;
        }
        let lib = plugin_loader::library_slot().lock().unwrap();
        let Some(lib) = lib.as_ref() else {
            continue;
        };
        let curio = unsafe {
            // look for curio_init — if not found this .so isn't a curio app
            let f: Symbol<InitCurioFn> = if let Ok(f) = lib.get(b"curio_init") { f } else { continue };
            // a plugin that panics in curio_init must not take the runner
            // down with it — catch, report, and skip this library
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(gpu))) {
                Ok(raw) if !raw.is_null() => Box::from_raw(raw),
                Ok(_) => {
                    eprintln!("curio_init null for {:?}", path);
                    continue;
                }
                Err(payload) => {
                    let msg = payload
                        .downcast_ref::<String>()
                        .cloned()
                        .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
                        .unwrap_or_else(|| "panic with no message".to_string());
                    eprintln!("CURIO PLUGIN PANIC in {:?}:\n  {msg}\n  (the application crashed while initializing)", path);
                    continue;
                }
            }
        };
        eprintln!("loaded: {}", curio.identity.name);
        return LoadedCurio { curio };
    }
    panic!("No curio plugin found");
}
