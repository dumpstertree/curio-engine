use std::sync::{Arc, Mutex};

use wgpu::{Adapter, Device, Instance, Queue, Surface};
use winit::event_loop::EventLoop;
use winit::window::{Fullscreen, Window};

use crate::IO::texture_asset::Texture_asset;
pub static SYS_GPU: Mutex<SystemGPU> = Mutex::new(SystemGPU {
    device: None,
    queue: None,
    surface: None,
    instance: None,
    adapter: None,
    window: None,
    depth_texture: None,
    config: None,
    arc_device: None,
});
pub enum CustomEvents {}
pub struct SystemGPU {
    pub arc_device: Option<Arc<Device>>,
    pub device: Option<Device>,
    pub queue: Option<Queue>,
    pub surface: Option<Surface<'static>>,
    pub instance: Option<Instance>,
    pub adapter: Option<Adapter>,
    pub window: Option<Arc<Window>>,
    pub depth_texture: Option<Texture_asset>,
    pub config: Option<wgpu::SurfaceConfiguration>,
}
impl SystemGPU {
    pub fn get_window() -> Arc<Window> {
        let s = SYS_GPU.lock().unwrap();
        match s.window.clone() {
            Some(x) => {
                drop(s);
                return x;
            }
            None => panic!("NOT SET"),
        }
    }
    pub fn get_device() -> Arc<Device> {
        let s = SYS_GPU.lock().unwrap();
        match s.arc_device.clone() {
            Some(x) => return x,
            None => panic!("NOT SET"),
        }
    }
    pub fn set_cursor_visible(visible: bool) {
        let sys = SYS_GPU.lock().unwrap();

        let Some(window) = &sys.window else {
            return;
        };

        window.set_cursor_visible(visible);
    }
    pub fn set_resizable(resizable: bool) {
        let sys = SYS_GPU.lock().unwrap();

        let Some(window) = &sys.window else {
            return;
        };
        window.set_resizable(resizable);
    }
    pub fn set_resolution(w: i32, h: i32) {
        let mut sys = SYS_GPU.lock().unwrap();

        let Some(device) = &sys.device else {
            return;
        };
        let Some(surface) = &sys.surface else {
            return;
        };

        let Some(config) = &sys.config else {
            return;
        };
        let Some(window) = &sys.window else {
            return;
        };

        let mut config = config.clone();
        config.width = w as u32;
        config.height = h as u32;

        let mut s = window.inner_size();
        s.width = w as u32;
        s.height = h as u32;

        println!("size {}, {} ", s.width, s.height);

        window.set_resizable(true);
        window.set_min_inner_size(Some(s));
        window.set_max_inner_size(Some(s));

        surface.configure(device, &config);

        sys.depth_texture = Some(Texture_asset::create_depth_texture(&device, &config, "depth_texture"));
        sys.config = Some(config);
    }
    pub fn set_fullscreen(fullscreeen: bool) {
        let sys = SYS_GPU.lock().unwrap();

        let Some(window) = &sys.window else {
            return;
        };

        if fullscreeen {
            window.set_fullscreen(Some(Fullscreen::Borderless(window.primary_monitor())));
            window.set_blur(true);
        } else {
            window.set_fullscreen(None);
            window.set_blur(false);
        }
    }
    pub async fn init(&mut self) -> EventLoop<CustomEvents> {
        let mut window_attributes = winit::window::Window::default_attributes();
        let event_loop: EventLoop<CustomEvents> = EventLoop::with_user_event().build().unwrap();
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
        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: 1920 as u32,
            height: 1080 as u32,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let d = Texture_asset::create_depth_texture(&device, &config, "depth_texture");
        self.surface = Some(surface);
        self.instance = Some(instance);
        self.device = Some(device);
        self.queue = Some(queue);
        self.adapter = Some(adapter);
        self.window = Some(window);
        self.depth_texture = Some(d);
        self.config = Some(config);
        // self.arc_device =
        event_loop
    }
}
