use std::{
    cell::RefCell,
    path::Path,
    sync::{Arc, Mutex},
};

use egui_wgpu::wgpu::{Adapter, Device, DeviceDescriptor, Features, Instance, Limits, PowerPreference, Queue, RequestAdapterOptions, Surface, SurfaceConfiguration, TextureFormat, TextureUsages};
use pollster::FutureExt;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::Window,
};

use curio_core::{
    built_in::record::{
        sys_record_camera::SysRecordCamera, sys_record_debug::SysRecordDebug, sys_record_debug_gui::SysRecordDebugGui, sys_record_gizmos::SysRecordGizmos, sys_record_gui::SysRecordGui, sys_record_input::SysRecordInput, sys_record_lights::SysRecordLights, sys_record_network::SysRecordNetwork,
        sys_record_rendering::SysRecordRendering, sys_record_screen::SysRecordScreen, sys_record_skybox::SysRecordSkybox, sys_record_sun::SysRecordSun, sys_record_time::SysRecordTime,
    },
    collections::{curio_metadata::CurioMetadata, version_number::VersionNumber, window_layout::WindowLayout},
    engine::{
        curio::{load_curio, Curio, LoadedCurio},
        curio_common::CurioCommon,
    },
    engine_services::{EngineServices, GpuHandle},
    static_data,
    system_adapters::adapter_system_gpu::SystemGPU,
    Application, AxisCode, ButtonCode, GPUInstance, KeyState, Severity, TextureAsset, Vector3,
};

static mut OPEN_DISPLAY_WINDOWS: Mutex<Vec<CabinetWindow>> = Mutex::new(Vec::new());

pub struct CurioCabinet {}
impl CurioCabinet {
    pub fn on_display() -> Vec<CabinetWindow> {
        // if let Ok(meta) = unsafe { OPEN_DISPLAY_WINDOWS.lock() } {
        //     return meta.to_vec();
        // }

        return Vec::new();
    }

    pub fn put_on_display() {
        //
        // register_built_in_records();

        //
        Application::log(Severity::Info, "Putting Curio on display...");

        // add to list of windows
        let window_owner = CabinetWindowOwner::new(WindowLayout::fullscreen_1080());

        // wrap the window so its easily sharable
        let mut window = CabinetWindow::new(window_owner);

        // store
        if let Ok(mut open_windows) = unsafe { OPEN_DISPLAY_WINDOWS.lock() } {
            open_windows.push(window.clone());
        }

        // run the window
        window.run();
    }
}

// pub fn register_built_in_records() {
//     static_data::global_states::register_global_state::<SysRecordTime>();
//     static_data::global_states::register_global_state::<SysRecordCamera>();
//     static_data::global_states::register_global_state::<SysRecordDebug>();
//     static_data::global_states::register_global_state::<SysRecordRendering>();
//     static_data::global_states::register_global_state::<SysRecordGizmos>();
//     static_data::global_states::register_global_state::<SysRecordDebugGui>();
//     static_data::global_states::register_global_state::<SysRecordGui>();
//     static_data::global_states::register_global_state::<SysRecordInput>();
//     static_data::global_states::register_global_state::<SysRecordLights>();
//     static_data::global_states::register_global_state::<SysRecordNetwork>();
//     static_data::global_states::register_global_state::<SysRecordScreen>();
//     static_data::global_states::register_global_state::<SysRecordSkybox>();
//     static_data::global_states::register_global_state::<SysRecordSun>();
// }
// curio_core
pub struct CabinetWindowOwner {
    did_run: bool,
    gpu_instance: Option<Arc<GPUInstance>>,
    gpu: Option<Arc<SystemGPU>>,           // ← new, keeps Arc alive
    services: Option<Box<EngineServices>>, // ← new, Box gives stable address
    // loaded_curio: Option<LoadedCurio>,     // ← new, keeps .so alive
    app_instance: Option<LoadedCurio>,
    portal: WindowLayout,
}

impl CabinetWindowOwner {
    pub fn new(portal: WindowLayout) -> CabinetWindowOwner {
        CabinetWindowOwner {
            did_run: false,
            gpu_instance: None,
            gpu: None,
            services: None,
            // loaded_curio: None,
            app_instance: None,
            portal,
        }
    }
}
impl CabinetWindowOwner {
    // pub fn new(/*curio: fn(*mut EngineServices) -> Box<Curio,*/ portal: WindowLayout) -> CabinetWindowOwner {
    //     println!("Created window");
    //     CabinetWindowOwner {
    //         did_run: false,
    //         gpu_instance: None,
    //         app_instance: None,
    //         // app_constructor: curio,
    //         portal: portal,
    //     }
    // }
    pub fn run(&mut self) {
        // guard - dont run if already running
        if self.did_run {
            return;
        }

        // enable flag
        self.did_run = true;

        // build an event loop to base everthing off of
        let Ok(event_loop) = EventLoop::builder().build() else {
            panic!("Failed to build an EventLoop parent for Curio");
        };

        // run the app. this will continue to loop
        let Ok(_result) = event_loop.run_app(self) else {
            panic!("Failed to run Curio");
        };
    }
    pub fn set_curio_instance(&mut self, curio_instance: LoadedCurio) {
        // save instance
        self.app_instance = Some(curio_instance);

        // window has now been opened and alert the app
        if let Some(app_instance) = &mut self.app_instance {
            app_instance.curio.window_opened();
        }
    }
    pub fn get_gpu_settings(&self) -> Arc<GPUInstance> {
        if let Some(gpu_settings) = &self.gpu_instance {
            gpu_settings.clone()
        } else {
            panic!("Failed to retrieve GPU");
        }
    }

    fn get_instance() -> Instance {
        let instance = egui_wgpu::wgpu::Instance::new(&egui_wgpu::wgpu::InstanceDescriptor {
            backends: egui_wgpu::wgpu::Backends::all(),
            ..Default::default()
        });

        instance
    }
    fn get_window(event_loop: &ActiveEventLoop, meta: &CurioMetadata, portal: &WindowLayout) -> Arc<Window> {
        // populate all the attributes to spawn the window
        let atts = Window::default_attributes()
            .with_title(format!(" {} - {}.{}.{}", meta.name, meta.version.major, meta.version.minor, meta.version.patch))
            .with_inner_size(PhysicalSize::new(portal.width, portal.height))
            .with_resizable(true);

        // create the window we are using
        let Ok(window) = event_loop.create_window(atts) else {
            panic!("Failed to create Window");
        };

        Arc::new(window)
    }
    fn get_surface(instance: &Instance, window: &Arc<Window>) -> Arc<Surface<'static>> {
        let surface = instance.create_surface(window.clone()).unwrap();
        Arc::new(surface)
    }
    fn get_adapter(instance: &Instance, surface: &Arc<Surface>) -> Arc<Adapter> {
        let Some(adapter) = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .block_on()
        else {
            panic!("Failed to create Adapter");
        };

        Arc::new(adapter)
    }
    fn get_device_queue(adapter: &Arc<Adapter>) -> (Arc<Device>, Arc<Queue>) {
        let Ok(device_queue) = adapter
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
        else {
            panic!("Failed to create Device or Queue");
        };
        (Arc::new(device_queue.0), Arc::new(device_queue.1))
    }
    fn get_config(surface: &Surface, adapter: &Adapter, portal: &WindowLayout) -> Arc<SurfaceConfiguration> {
        let surface_capabilities = surface.get_capabilities(&adapter);

        let surface_format: TextureFormat = surface_capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_capabilities.formats[0]);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: portal.width as u32,
            height: portal.height as u32,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        Arc::new(config)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn set_resolution(_x: i32, _y: i32) {}
#[unsafe(no_mangle)]
pub extern "C" fn set_fullscreen(_x: bool) {}
#[unsafe(no_mangle)]
pub extern "C" fn set_cursor_visible(_x: bool) {}
impl ApplicationHandler for CabinetWindowOwner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let instance = Self::get_instance();
        let window = Self::get_window(event_loop, &CurioMetadata::new("", "", VersionNumber::new(0, 0, 0)), &self.portal);
        let surface = Self::get_surface(&instance, &window);
        let adapter = Self::get_adapter(&instance, &surface);
        let (device, queue) = Self::get_device_queue(&adapter);
        let config = Self::get_config(&surface, &adapter, &self.portal);
        let depth_texture = Arc::new(create_depth_texture(&device, "DEPTH"));

        self.gpu_instance = Some(Arc::new(GPUInstance {
            device,
            queue,
            surface,
            adapter,
            window,
            config,
            depth_texture: depth_texture.clone(),
        }));

        // gpu lives on self — Arc won't drop until CabinetWindowOwner drops
        let gpu = Arc::new(SystemGPU::new(
            self.gpu_instance.as_ref().unwrap().clone(), // clone the Arc, don't move
        ));

        // services is Boxed so it has a stable heap address
        // the Box lives on self — won't drop until CabinetWindowOwner drops
        self.services = Some(Box::new(EngineServices {
            gpu: GpuHandle {
                device: Arc::as_ptr(&gpu.device) as *const (),
                queue: Arc::as_ptr(&gpu.queue) as *const (),
                config: Arc::as_ptr(&gpu.config) as *const (),
                window: Arc::as_ptr(&gpu.window) as *const (),
                surface: Arc::as_ptr(&gpu.surface) as *const (),
                depth: Arc::as_ptr(&depth_texture) as *const (),
                capture_texture: todo!(),
                capture_width: todo!(),
                capture_height: todo!(),
            },
            set_fullscreen: set_fullscreen,
            set_resolution: set_resolution,
            set_cursor_visible: set_cursor_visible,
        }));

        self.gpu = Some(gpu);

        // get a raw pointer to the Boxed services — stable as long as Box lives
        let services_ptr: *const EngineServices = self.services.as_deref().unwrap();

        // keep LoadedCurio on self so the .so stays loaded
        let mut loaded = load_curio(Path::new("./plugins"), services_ptr);

        loaded.curio.window_opened();

        // self.app_instance = Some(loaded.curio);
        self.app_instance = Some(loaded); // ← Library kept alive here
    }
    fn window_event(&mut self, _: &winit::event_loop::ActiveEventLoop, _: winit::window::WindowId, event: winit::event::WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                if let Some(app_instance) = &mut self.app_instance {
                    app_instance.curio.application_refresh();
                }

                if let Some(x) = &self.gpu_instance {
                    x.window.request_redraw()
                };
            }
            WindowEvent::Resized(_physical_size) => {
                if let Some(app_instance) = &mut self.app_instance {
                    app_instance.curio.window_resized();
                }
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
                        // get the current button state to propogate
                        let state = if event.state.is_pressed() { KeyState::Down } else { KeyState::Up };

                        // if button was resolved we propogate the event
                        app_instance.curio.input_button(button_code, state);
                    };
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
                    // if button was resolved we propogate the event
                    if let Some(button_code) = ButtonCode::from_winit_mousebutton(button) {
                        // get the current button state to propogate
                        let state = if state.is_pressed() { KeyState::Down } else { KeyState::Up };

                        // if button was resolved we propogate the event
                        app_instance.curio.input_button(button_code, state);
                    };
                }
            }

            _ => {}
        }
    }
}

#[derive(Clone)]
pub struct CabinetWindow {
    owner: Arc<RefCell<CabinetWindowOwner>>,
}
impl CabinetWindow {
    pub fn new(window: CabinetWindowOwner) -> CabinetWindow {
        CabinetWindow { owner: Arc::new(RefCell::new(window)) }
    }
    pub fn run(&mut self) {
        self.owner.borrow_mut().run();
    }
    // pub fn curio(&self) -> CurioMetadata {
    //     self.owner
    //         .borrow()
    //         .app_instance
    //         .as_ref()
    //         .unwrap()
    //         .meta
    //         .clone()
    // }
}

pub fn create_depth_texture(device: &Device, label: &str) -> TextureAsset {
    // let config = gpu.config;
    let size = egui_wgpu::wgpu::Extent3d {
        width: 1920,
        height: 1080,
        // width: config.width.max(1),
        // height: config.height.max(1),
        depth_or_array_layers: 1,
    };

    // println!("create depth with size {}, {}", config.width.max(1), config.height.max(1));
    let desc = egui_wgpu::wgpu::TextureDescriptor {
        label: Some(label),
        mip_level_count: 1,
        sample_count: 1,
        dimension: egui_wgpu::wgpu::TextureDimension::D2,
        format: TextureAsset::DEPTH_FORMAT,
        usage: egui_wgpu::wgpu::TextureUsages::RENDER_ATTACHMENT // 3.
                |  egui_wgpu::wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
        size,
    };

    let texture = device.create_texture(&desc);
    let view = texture.create_view(&egui_wgpu::wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&egui_wgpu::wgpu::SamplerDescriptor {
        // 4.
        address_mode_u: egui_wgpu::wgpu::AddressMode::ClampToEdge,
        address_mode_v: egui_wgpu::wgpu::AddressMode::ClampToEdge,
        address_mode_w: egui_wgpu::wgpu::AddressMode::ClampToEdge,
        mag_filter: egui_wgpu::wgpu::FilterMode::Linear,
        min_filter: egui_wgpu::wgpu::FilterMode::Linear,
        mipmap_filter: egui_wgpu::wgpu::FilterMode::Nearest,
        compare: Some(egui_wgpu::wgpu::CompareFunction::LessEqual), // 5.
        lod_min_clamp: 0.0,
        lod_max_clamp: 100.0,
        ..Default::default()
    });

    TextureAsset {
        // width: size.width as i32,
        // height: size.height as i32,
        sampler: sampler,
        texture: texture,
        view: view,
    }
}
