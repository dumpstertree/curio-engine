use std::sync::{Arc, Mutex};

use egui_wgpu::wgpu::{Adapter, Device, Queue, Surface, SurfaceConfiguration};
use winit::dpi::PhysicalSize;
use winit::window::{Fullscreen, Window};

use crate::dumpster_engine::GPUInstance;
use crate::io::texture_asset::TextureAsset;
pub static SYSTEM_GPU_ADAPTER_INSTANCE: Mutex<SystemGPU> = Mutex::new(SystemGPU {
    device: None,
    queue: None,
    surface: None,
    adapter: None,
    window: None,
    depth_texture: None,
    config: None,
});

pub struct SystemGPU {
    device: Option<Arc<Device>>,
    queue: Option<Arc<Queue>>,
    surface: Option<Arc<Surface<'static>>>,
    adapter: Option<Arc<Adapter>>,
    window: Option<Arc<Window>>,
    depth_texture: Option<Arc<TextureAsset>>,
    config: Option<Arc<SurfaceConfiguration>>,
}
impl SystemGPU {
    pub fn get_device() -> Arc<Device> {
        let Ok(guard) = SYSTEM_GPU_ADAPTER_INSTANCE.lock() else {
            panic!("FAILED");
        };
        match &guard.device {
            Some(x) => return x.clone(),
            None => panic!("NO DEVICE"),
        }
    }
    pub fn get_queue() -> Arc<Queue> {
        let Ok(guard) = SYSTEM_GPU_ADAPTER_INSTANCE.lock() else {
            panic!("FAILED");
        };
        match &guard.queue {
            Some(x) => return x.clone(),
            None => panic!("NO QUEUE"),
        }
    }
    pub fn get_surface() -> Arc<Surface<'static>> {
        let Ok(guard) = SYSTEM_GPU_ADAPTER_INSTANCE.lock() else {
            panic!("FAILED");
        };
        match &guard.surface {
            Some(x) => return x.clone(),
            None => panic!("NO SURFACE"),
        }
    }
    pub fn get_adapter() -> Arc<Adapter> {
        let Ok(guard) = SYSTEM_GPU_ADAPTER_INSTANCE.lock() else {
            panic!("FAILED");
        };
        match &guard.adapter {
            Some(x) => return x.clone(),
            None => panic!("NO APADTER"),
        }
    }
    pub fn get_window() -> Arc<Window> {
        let Ok(guard) = SYSTEM_GPU_ADAPTER_INSTANCE.lock() else {
            panic!("FAILED");
        };
        match &guard.window {
            Some(x) => return x.clone(),
            None => panic!("NO WINDOW"),
        }
    }
    pub fn get_depth_texture() -> Arc<TextureAsset> {
        let Ok(guard) = SYSTEM_GPU_ADAPTER_INSTANCE.lock() else {
            panic!("FAILED");
        };
        match &guard.depth_texture {
            Some(x) => return x.clone(),
            None => panic!("NO DEPTH"),
        }
    }
    pub fn get_config() -> Arc<SurfaceConfiguration> {
        let Ok(guard) = SYSTEM_GPU_ADAPTER_INSTANCE.lock() else {
            panic!("FAILED");
        };
        match &guard.config {
            Some(x) => return x.clone(),
            None => panic!("NO DEVICE"),
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

        let mut config: SurfaceConfiguration;
        {
            let c = SystemGPU::get_config();
            let surface = SystemGPU::get_surface();
            let window = SystemGPU::get_window();
            let device = SystemGPU::get_device();

            config = (*c).clone();
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
            println!("surface configured")
        }
        {
            // let dt = Some(Arc::new(TextureAsset::create_depth_texture("depth_texture")));
            let Ok(mut guard) = SYSTEM_GPU_ADAPTER_INSTANCE.lock() else {
                panic!("FAILED");
            };
            // guard.depth_texture = dt;
            guard.config = Some(Arc::new(config));
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

    //
    pub fn set_global_values(gpu_instance: Arc<GPUInstance>) {
        {
            // guard -
            let Ok(mut guard) = SYSTEM_GPU_ADAPTER_INSTANCE.lock() else {
                panic!("Failed to lock GPU Instance");
            };

            println!("{:#?}", gpu_instance.adapter.get_info());

            //
            guard.surface = Some(gpu_instance.surface.clone());
            guard.device = Some(gpu_instance.device.clone());
            guard.queue = Some(gpu_instance.queue.clone());
            guard.adapter = Some(gpu_instance.adapter.clone());
            guard.window = Some(gpu_instance.window.clone());
            guard.config = Some(gpu_instance.config.clone());
        }
        {
            let dt = Some(Arc::new(TextureAsset::create_depth_texture("depth_texture")));

            // guard -
            let Ok(mut guard) = SYSTEM_GPU_ADAPTER_INSTANCE.lock() else {
                panic!("Failed to lock GPU Instance");
            };
            guard.depth_texture = dt;
        }
        {
            // guard -
            let Ok(mut guard) = SYSTEM_GPU_ADAPTER_INSTANCE.lock() else {
                panic!("Failed to lock GPU Instance");
            };

            let Some(surface) = &guard.surface else {
                return;
            };
            let Some(device) = &guard.device else {
                return;
            };
            let Some(config) = &guard.config else {
                return;
            };

            surface.configure(&(*device), &config);
        }
    }
}
