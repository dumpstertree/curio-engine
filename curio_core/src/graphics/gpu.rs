use egui_wgpu::wgpu::{Device, Queue, Texture};

/// Access to the GPU and GPU params on the host system
#[repr(C)]
pub struct Gpu {
    pub device: *const (),
    pub queue: *const (),
    pub capture_texture: *const (),
    pub capture_width: u32,
    pub capture_height: u32,
}
impl Gpu {
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
