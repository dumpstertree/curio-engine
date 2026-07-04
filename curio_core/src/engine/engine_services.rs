use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

// use egui::Window;
use crate::io::asset_loader::AssetLoader;
use crate::io::log::Logger;
use egui_wgpu::wgpu::{Device, Queue, Texture};

#[repr(C)]
pub struct GpuHandle {
    pub device: *const (),
    pub queue: *const (),
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
    pub fn capture_texture(&self) -> Option<&Texture> {
        if self.capture_texture.is_null() {
            None
        } else {
            Some(unsafe { &*(self.capture_texture as *const Texture) })
        }
    }
}

unsafe impl Send for EngineServices {}
unsafe impl Sync for EngineServices {}
#[repr(C)]
pub struct EngineServices {
    pub logger: *mut Logger,
    pub assets: *mut AssetLoader,
    pub gpu: GpuHandle,
    // pub set_resolution: unsafe extern "C" fn(w: i32, h: i32),
    // pub set_fullscreen: unsafe extern "C" fn(fullscreen: bool),
    // pub set_cursor_visible: unsafe extern "C" fn(visible: bool),
}
impl EngineServices {
    // pub fn set_resolution2(&mut self, w: u32, h: u32) {
    //     println!("try set resolution {} x {}", w, h)
    // }
    pub fn logger(&self) -> &mut Logger {
        unsafe { &mut *self.logger }
    }
    pub fn assets(&self) -> &mut AssetLoader {
        unsafe { &mut *self.assets }
    }
}

// the one static — safe to duplicate because both copies
// get set to the same pointer value at init
static SERVICES: AtomicPtr<EngineServices> = AtomicPtr::new(ptr::null_mut());

pub fn set_services(ptr: *const EngineServices) {
    println!("Services Set!");
    SERVICES.store(ptr as *mut _, Ordering::SeqCst);
}

pub fn services() -> &'static EngineServices {
    let ptr = SERVICES.load(Ordering::SeqCst);
    assert!(!ptr.is_null(), "EngineServices not initialised — was curio_init called?");
    unsafe { &*ptr }
}
