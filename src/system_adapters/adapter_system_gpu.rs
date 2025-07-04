use std::sync::{Arc, Mutex};

use wgpu::{Adapter, Device, Instance, Queue, Surface};
use winit::event_loop::EventLoop;
use winit::window::Window;

use crate::Window::state::State;
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
