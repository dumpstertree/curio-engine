use std::sync::{Arc, Mutex};

use egui_wgpu::wgpu::{Adapter, Device, DeviceDescriptor, Features, Instance, Limits, PowerPreference, Queue, RequestAdapterOptions, Surface, SurfaceConfiguration, TextureFormat, TextureUsages};
use pollster::FutureExt;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::Window,
};

use crate::{
    built_in::record::{
        state_camera::CameraState, state_debug::StateDebug, state_draw::DrawCallsState, state_gizmos::GizmosState, state_gui::GUIState, state_gui_debug::GUIStateDebug, state_input::InputState, state_lights::StateLights, state_network::StateNetwork, state_screeen::StateScreen,
        state_skybox::StateSkybox, state_sun::StateSun, state_time::TimeState,
    },
    collections::{key_state::KeyState, vector3::Vector3},
    dumpster_engine::{CurioMetadata, GPUInstance, WindowLayout},
    engine::curio_common::CurioCommon,
    input::{axis_code::AxisCode, key_code::ButtonCode},
    static_data,
    system_adapters::adapter_system_gpu::SystemGPU,
};

static mut ALL_META: Mutex<Vec<CurioMetadata>> = Mutex::new(Vec::new());

pub fn curios_on_display() -> Vec<CurioMetadata> {
    if let Ok(meta) = unsafe { ALL_META.lock() } {
        return meta.clone();
    }

    return Vec::new();
}

pub struct CurioCabinet<T>
where
    T: 'static + CurioCommon,
{
    gpu_instance: Option<Arc<GPUInstance>>,
    app_instance: Option<Box<dyn CurioCommon>>,
    app_constructor: fn() -> T,
    portal: WindowLayout,
    meta: CurioMetadata,
}
impl<T> CurioCabinet<T>
where
    T: 'static + CurioCommon,
{
    pub fn display_curio(meta: CurioMetadata, curio: fn() -> T, portal: WindowLayout)
    // add a metadata object - Name, Version Num, IconPath
    where
        T: 'static + CurioCommon,
    {
        register_built_in_records();
        // store this curio metadata
        if let Ok(mut m) = unsafe { ALL_META.lock() } {
            m.push(meta.clone());
        }

        // create a new curio_engine instance
        let mut curio_engine = CurioCabinet {
            app_instance: None,
            gpu_instance: None,
            app_constructor: curio,
            portal,
            meta,
        };

        // build an event loop to base everthing off of
        let Ok(event_loop) = EventLoop::builder().build() else {
            panic!("Failed to build an EventLoop parent for Curio");
        };

        // run the app. this will continue to loop
        let Ok(_result) = event_loop.run_app(&mut curio_engine) else {
            panic!("Failed to run Curio");
        };
    }
}

// Impl - Public Fns
impl<T> CurioCabinet<T>
where
    T: 'static + CurioCommon,
{
    pub fn set_curio_instance(&mut self, curio_instance: T) {
        // save instance
        self.app_instance = Some(Box::new(curio_instance));

        // window has now been opened and alert the app
        if let Some(app_instance) = &mut self.app_instance {
            app_instance.window_opened();
        }
    }
    pub fn get_gpu_settings(&self) -> Arc<GPUInstance> {
        if let Some(gpu_settings) = &self.gpu_instance {
            gpu_settings.clone()
        } else {
            panic!("Failed to retrieve GPU");
        }
    }
}

// Impl - Private Fns
impl<T> CurioCabinet<T>
where
    T: 'static + CurioCommon,
{
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

impl<T> ApplicationHandler for CurioCabinet<T>
where
    T: 'static + CurioCommon,
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // generate an instance of wgpu to create rendering components
        let instance = &Self::get_instance();

        // generate the window. Window is the bounds and controller to the app.
        let window = Self::get_window(event_loop, &self.meta, &self.portal);

        // create the surface to render to
        let surface = Self::get_surface(instance, &window);

        // create the handle to the physical graphics
        let adapter = Self::get_adapter(instance, &surface);

        // create a connection to the device through the adapter and create a command queue to make changes to it
        let device_queue = Self::get_device_queue(&adapter);

        // find the format we are going to write our texture to based on the capabilities
        let surface_configuration = Self::get_config(&surface, &adapter, &self.portal);

        // package values to represent this instance of wgpu to reference later
        self.gpu_instance = Some(Arc::new(GPUInstance {
            device: device_queue.0,
            queue: device_queue.1,
            surface: surface,
            adapter: adapter,
            window: window,
            config: surface_configuration,
        }));

        // now that the curio_engine is initialized use those values to populate the system
        SystemGPU::set_global_values(self.get_gpu_settings());

        // generate an instance of the curio that we will then put into the curio_engine
        // this need to take place after populating the gpu settings incase something uses them
        self.app_instance = Some(Box::new((self.app_constructor)()));

        // assuming the app instance was created successfully we trigger window opened - the start of the application logic
        if let Some(app_instance) = &mut self.app_instance {
            app_instance.window_opened();
        };
    }

    fn window_event(&mut self, _: &winit::event_loop::ActiveEventLoop, _: winit::window::WindowId, event: winit::event::WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                if let Some(app_instance) = &mut self.app_instance {
                    app_instance.application_refresh();
                }

                if let Some(x) = &self.gpu_instance {
                    x.window.request_redraw()
                };
            }
            WindowEvent::Resized(_physical_size) => {
                if let Some(app_instance) = &mut self.app_instance {
                    app_instance.window_resized();
                }
            }
            WindowEvent::Moved(_physical_position) => {
                if let Some(app_instance) = &mut self.app_instance {
                    app_instance.window_moved();
                }
            }
            WindowEvent::Focused(is_focused) => {
                if let Some(app_instance) = &mut self.app_instance {
                    app_instance.window_focused(is_focused);
                }
            }
            WindowEvent::Occluded(is_occluded) => {
                if let Some(app_instance) = &mut self.app_instance {
                    app_instance.window_occluded(is_occluded);
                }
            }
            WindowEvent::KeyboardInput { device_id: _, event, is_synthetic: _ } => {
                if let Some(app_instance) = &mut self.app_instance {
                    if let Some(button_code) = ButtonCode::from_winit_physical_key(event.physical_key) {
                        // get the current button state to propogate
                        let state = if event.state.is_pressed() { KeyState::Down } else { KeyState::Up };

                        // if button was resolved we propogate the event
                        app_instance.input_button(button_code, state);
                    };
                }
            }
            WindowEvent::CursorMoved { device_id: _, position } => {
                if let Some(app_instance) = &mut self.app_instance {
                    app_instance.input_axis(AxisCode::Cursor, Vector3::new(position.x as f32, position.y as f32, 0.0));
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
                        app_instance.input_button(button_code, state);
                    };
                }
            }

            _ => {}
        }
    }
}

pub fn register_built_in_records() {
    static_data::global_states::register_global_state::<TimeState>();
    static_data::global_states::register_global_state::<CameraState>();
    static_data::global_states::register_global_state::<StateDebug>();
    static_data::global_states::register_global_state::<DrawCallsState>();
    static_data::global_states::register_global_state::<GizmosState>();
    static_data::global_states::register_global_state::<GUIStateDebug>();
    static_data::global_states::register_global_state::<GUIState>();
    static_data::global_states::register_global_state::<InputState>();
    static_data::global_states::register_global_state::<StateLights>();
    static_data::global_states::register_global_state::<StateNetwork>();
    static_data::global_states::register_global_state::<StateScreen>();
    static_data::global_states::register_global_state::<StateSkybox>();
    static_data::global_states::register_global_state::<StateSun>();
}
