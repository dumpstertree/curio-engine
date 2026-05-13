use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

// use egui::Window;
use egui_wgpu::wgpu::{Device, Queue, Surface, SurfaceConfiguration, Texture};
use winit::window::Window;

use crate::TextureAsset;

#[repr(C)]
pub struct GpuHandle {
    pub surface: *const (),
    pub device: *const (),
    pub queue: *const (),
    pub config: *const (),
    pub window: *const (),
    pub depth: *const (),
    pub capture_texture: *const (),
    pub capture_width: u32,
    pub capture_height: u32,
}

impl GpuHandle {
    pub fn device(&self) -> &Device {
        unsafe { &*(self.device as *const Device) }
    }
    pub fn queue(&self) -> &Queue {
        unsafe { &*(self.queue as *const Queue) }
    }
    pub fn config(&self) -> &SurfaceConfiguration {
        unsafe { &*(self.config as *const SurfaceConfiguration) }
    }
    pub fn window(&self) -> &Window {
        unsafe { &*(self.window as *const Window) }
    }
    pub fn depth(&self) -> &TextureAsset {
        unsafe { &*(self.depth as *const TextureAsset) }
    }
    pub fn surface(&self) -> &Surface<'_> {
        unsafe { &*(self.surface as *const Surface) }
    }
    pub fn capture_texture(&self) -> Option<&Texture> {
        if self.capture_texture.is_null() {
            None
        } else {
            Some(unsafe { &*(self.capture_texture as *const Texture) })
        }
    }
}
#[repr(C)]
pub struct EngineServices {
    pub gpu: GpuHandle,
    pub set_resolution: unsafe extern "C" fn(w: i32, h: i32),
    pub set_fullscreen: unsafe extern "C" fn(fullscreen: bool),
    pub set_cursor_visible: unsafe extern "C" fn(visible: bool),
}

// the one static — safe to duplicate because both copies
// get set to the same pointer value at init
static SERVICES: AtomicPtr<EngineServices> = AtomicPtr::new(ptr::null_mut());

pub fn set_services(ptr: *const EngineServices) {
    SERVICES.store(ptr as *mut _, Ordering::SeqCst);
}

pub fn services() -> &'static EngineServices {
    let ptr = SERVICES.load(Ordering::SeqCst);
    assert!(!ptr.is_null(), "EngineServices not initialised — was curio_init called?");
    unsafe { &*ptr }
}
