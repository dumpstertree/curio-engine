use std::any::Any;
use std::os::linux::raw::stat;
use std::sync::Arc;

use pollster::FutureExt;
use wgpu::wgc::device::{self, queue};
use wgpu::{Adapter, Device, Instance, Queue, Surface, SurfaceConfiguration};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::{self, Window};
use winit::{application::ApplicationHandler, event::WindowEvent};

use crate::game_state::{self, GameState};
use crate::system::system_component::ISystemComponent;
use crate::system::system_components::graphics_component::IGraphicsComponent;
use crate::Collections::key_state::KeyState;
use crate::Window::state::{self, State};

use std::sync::Mutex;
pub static SYS_GPU: Mutex<SystemGPU> = Mutex::new(SystemGPU {
    device: None,
    queue: None,
    surface: None,
    instance: None,
    adapter: None,
    window: None,
});

pub struct SystemGPU {
    pub device: Option<Device>,
    pub queue: Option<Queue>,
    pub surface: Option<Surface<'static>>,
    pub instance: Option<Instance>,
    pub adapter: Option<Adapter>,
    pub window: Option<Arc<Window>>,
}
impl SystemGPU {
    pub async fn from_window(&mut self) -> EventLoop<State> {
        let mut window_attributes = winit::window::Window::default_attributes();
        let event_loop: EventLoop<State> = EventLoop::with_user_event().build().unwrap();
        let window: Arc<Window> = event_loop.create_window(window_attributes).unwrap().into();

        // let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        //
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        // setup the device
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off, // Trace path
            })
            .await
            .unwrap();
        self.surface = Some(surface);
        self.instance = Some(instance);
        self.device = Some(device);
        self.queue = Some(queue);
        self.adapter = Some(adapter);
        self.window = Some(window);

        event_loop
    }
}
pub struct SystemWindow {
    // assetloader: AssetLoader<'static>,
    gamestate: GameState,
    components: Vec<Box<dyn ISystemComponent>>,
    state: Option<State>,
}
impl SystemWindow {
    // constructor
    pub fn new(graphics: Box<dyn IGraphicsComponent>, components: Vec<Box<dyn ISystemComponent>>) -> SystemWindow {
        // window
        let mut s = SystemWindow {
            // assetloader: AssetLoader::new(ShaderCache::new(&state), &state),
            gamestate: GameState::new(),
            components: components,
            state: None,
        };

        // return
        s
    }

    // fn correct_stale_frame(state: &mut State) {
    //     let size = state.window.inner_size();
    //     SystemWindow::handle_resize(state, size.width, size.height);
    // }
    // fn handle_resize(state: &mut State, width: u32, height: u32) {
    //     if width > 0 && height > 0 {
    //         state.config.width = width;
    //         state.config.height = height;
    //         state.surface.configure(&state.box_device, &state.config);
    //         state.is_surface_configured = true;

    //         //Make sure you update the depth_texture after you update config. If you don't, your program will crash as the depth_texture will be a different size than the surface texture.
    //         state.depth_texture = super::super::texture::Texture::create_depth_texture(&state.box_device, &state.config, "depth_texture");
    //     }
    // }
    pub fn run(&mut self) {
        // create window
        // let mut window_attributes = winit::window::Window::default_attributes();
        // let event_loop = EventLoop::with_user_event().build().unwrap();
        // let window: Arc<Window> = event_loop.create_window(window_attributes).unwrap().into();

        // // init system gpu
        let mut guard_sys_gpu = SYS_GPU.lock().unwrap();
        let event_loop = guard_sys_gpu.from_window().block_on();
        drop(guard_sys_gpu);

        // self.gamestate.add(KEY_DEVICE_STATE, state);
        self.state = Some(state::State::new().block_on());
        let Some(state) = self.state.as_mut() else {
            return;
        };

        // drop the guard before initing components
        // drop(guard_sys_gpu);

        // surface.configure(device, config);
        // init all sub
        // sort based on order
        self.components.sort_by(|a, b| a.order().cmp(&b.order()));

        // iterate over each in new order
        for c in self.components.iter_mut() {
            c.init(state, &mut self.gamestate);
        }

        // run
        let _ = event_loop.run_app(self);
    }
}
impl ApplicationHandler<State> for SystemWindow {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        // reorder the compnents incase something changed
        self.components.sort_by(|a, b| a.order().cmp(&b.order()));

        // match the event
        match event {
            WindowEvent::CloseRequested => {
                for c in self.components.iter_mut() {
                    c.quit();
                }
            }
            WindowEvent::Resized(size) => {
                // let x = SYS_GPU.lock().unwrap()
                for c in self.components.iter_mut() {
                    c.resize(size.width as f32, size.height as f32);
                }
            }
            WindowEvent::RedrawRequested => {
                for c in self.components.iter_mut() {
                    match &mut self.state {
                        Some(state) => {
                            c.render(state, &mut self.gamestate);
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                for c in self.components.iter_mut() {
                    // convert to key code
                    let code;
                    match event.physical_key {
                        winit::keyboard::PhysicalKey::Code(key_code) => {
                            code = key_code;
                        }
                        _ => return,
                    }

                    // next
                    let state = if event.state.is_pressed() { KeyState::Down } else { KeyState::Up };
                    c.input_keyboard(&mut self.gamestate, code, state);
                }
            }
            WindowEvent::MouseInput { device_id, state, button } => {
                for c in self.components.iter_mut() {
                    c.input_mouse();
                }
            }
            _ => {}
        }
    }
}
