use std::sync::Arc;

use egui_wgpu::wgpu::{Adapter, Device, Instance, Queue, Surface, SurfaceConfiguration};
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Fullscreen, Window, WindowAttributes};

use crate::events::engine_commands::EngineCommands;
use crate::IO::texture_asset::Texture_asset;
pub static mut system_gpu_adapter_instance: SystemGPU = SystemGPU {
    device: None,
    queue: None,
    surface: None,
    instance: None,
    adapter: None,
    window: None,
    depth_texture: None,
    config: None,
};

// #[derive(Clone)]
// pub enum CustomEvents {
//     Tick,
// }
pub struct SystemGPU {
    device: Option<Arc<Device>>,
    queue: Option<Arc<Queue>>,
    surface: Option<Arc<Surface<'static>>>,
    instance: Option<Arc<Instance>>,
    adapter: Option<Arc<Adapter>>,
    window: Option<Arc<Window>>,
    depth_texture: Option<Arc<Texture_asset>>,
    config: Option<Arc<SurfaceConfiguration>>,
}
impl SystemGPU {
    pub fn get_device() -> Arc<Device> {
        unsafe {
            match &system_gpu_adapter_instance.device {
                Some(x) => return x.clone(),
                None => panic!("NO DEVICE"),
            }
        }
    }
    pub fn get_queue() -> Arc<Queue> {
        unsafe {
            match &system_gpu_adapter_instance.queue {
                Some(x) => return x.clone(),
                None => panic!("NO QUEUE"),
            }
        }
    }
    pub fn get_surface() -> Arc<Surface<'static>> {
        unsafe {
            match &system_gpu_adapter_instance.surface {
                Some(x) => return x.clone(),
                None => panic!("NO SURFACE"),
            }
        }
    }
    pub fn get_instance() -> Arc<Instance> {
        unsafe {
            match &system_gpu_adapter_instance.instance {
                Some(x) => return x.clone(),
                None => panic!("NO INSTANCE"),
            }
        }
    }
    pub fn get_adapter() -> Arc<Adapter> {
        unsafe {
            match &system_gpu_adapter_instance.adapter {
                Some(x) => return x.clone(),
                None => panic!("NO APADTER"),
            }
        }
    }
    pub fn get_window() -> Arc<Window> {
        unsafe {
            match &system_gpu_adapter_instance.window {
                Some(x) => return x.clone(),
                None => panic!("NO WINDOW"),
            }
        }
    }
    pub fn get_depth_texture() -> Arc<Texture_asset> {
        unsafe {
            match &system_gpu_adapter_instance.depth_texture {
                Some(x) => return x.clone(),
                None => panic!("NO DEVICE"),
            }
        }
    }
    pub fn get_config() -> Arc<SurfaceConfiguration> {
        unsafe {
            match &system_gpu_adapter_instance.config {
                Some(x) => return x.clone(),
                None => panic!("NO DEVICE"),
            }
        }
    }
    pub fn set_cursor_visible(visible: bool) {
        let window = SystemGPU::get_window();
        window.set_cursor_visible(visible);
    }
    pub fn set_resizable(resizable: bool) {
        let window = SystemGPU::get_window();
        window.set_resizable(resizable);
    }
    pub fn set_resolution(w: i32, h: i32) {
        println!("set {}, {}", w, h);
        let config = SystemGPU::get_config();
        let surface = SystemGPU::get_surface();
        let window = SystemGPU::get_window();
        let device = SystemGPU::get_device();

        let mut config = (*config).clone();
        config.width = w as u32;
        config.height = h as u32;

        // let mut s = window.inner_size();
        // s.width = w as u32;
        // s.height = h as u32;

        // println!("size {}, {} ", s.width, s.height);
        // window.set_resizable(true);
        // window.set_min_inner_size(Some(LogicalSize::new(config.width, config.height)));
        // window.set_max_inner_size(Some(LogicalSize::new(config.width, config.height)));
        let _ = window.request_inner_size(PhysicalSize::new(config.width, config.height));

        surface.configure(&(*device), &config);

        unsafe {
            system_gpu_adapter_instance.depth_texture = Some(Arc::new(Texture_asset::create_depth_texture("depth_texture")));
            system_gpu_adapter_instance.config = Some(Arc::new(config));
        }
    }
    pub fn set_fullscreen(fullscreeen: bool) {
        let window = SystemGPU::get_window();

        if fullscreeen {
            window.set_fullscreen(Some(Fullscreen::Borderless(window.primary_monitor())));
            window.set_blur(true);
        } else {
            window.set_fullscreen(None);
            window.set_blur(false);
        }
    }
    pub async fn init() -> EventLoop<EngineCommands> {
        let window_attributes = WindowAttributes::default()
            .with_inner_size(winit::dpi::LogicalSize::new(1920.0, 1080.0))
            .with_title("My Window");
        let event_loop: EventLoop<EngineCommands> = EventLoop::with_user_event().build().unwrap();
        let window: Arc<Window> = event_loop.create_window(window_attributes).unwrap().into();
        // let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = egui_wgpu::wgpu::Instance::new(&egui_wgpu::wgpu::InstanceDescriptor {
            backends: egui_wgpu::wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        //
        let adapter = instance
            .request_adapter(&egui_wgpu::wgpu::RequestAdapterOptions {
                power_preference: egui_wgpu::wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        // setup the device
        let (device, queue) = adapter
            .request_device(
                &egui_wgpu::wgpu::DeviceDescriptor {
                    label: None,
                    required_features: egui_wgpu::wgpu::Features::POLYGON_MODE_LINE,
                    //  features: (optional_features & adapter_features) | required_features,
                    // WebGL doesn't support all of egui_wgpu::wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    required_limits: egui_wgpu::wgpu::Limits::default(),
                    memory_hints: Default::default(),
                    // trace: egui_wgpu::wgpu::Trace::Off, // Trace path
                },
                None,
            )
            .await
            .unwrap();
        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = egui_wgpu::wgpu::SurfaceConfiguration {
            usage: egui_wgpu::wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: 1920 as u32,
            height: 1080 as u32,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        unsafe {
            system_gpu_adapter_instance = SystemGPU {
                surface: Some(Arc::new(surface)),
                instance: Some(Arc::new(instance)),
                device: Some(Arc::new(device)),
                queue: Some(Arc::new(queue)),
                adapter: Some(Arc::new(adapter)),
                window: Some(window),
                depth_texture: None,
                config: Some(Arc::new(config)),
            };

            system_gpu_adapter_instance.depth_texture = Some(Arc::new(Texture_asset::create_depth_texture("depth_texture")));
            // let d = Texture_asset::create_depth_texture("depth_texture");
        }
        event_loop
    }
}
