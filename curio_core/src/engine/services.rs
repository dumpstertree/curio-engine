use crate::io::asset_loader::AssetLoader;
use crate::io::log::Logger;
use crate::GpuHandle;
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

// the one static — safe to duplicate because both copies
// get set to the same pointer value at init
static SERVICES: AtomicPtr<Services> = AtomicPtr::new(ptr::null_mut());

#[repr(C)]
pub struct Services {
    pub logger: *mut Logger,
    pub assets: *mut AssetLoader,
    pub gpu: GpuHandle,
}
impl Services {
    pub fn set(ptr: *const Services) {
        SERVICES.store(ptr as *mut _, Ordering::SeqCst);
    }
    pub fn get() -> &'static Services {
        let ptr = SERVICES.load(Ordering::SeqCst);
        assert!(!ptr.is_null(), "EngineServices not initialised — was curio_init called?");
        unsafe { &*ptr }
    }
}
impl Services {
    pub fn logger(&self) -> &mut Logger {
        unsafe { &mut *self.logger }
    }
    pub fn assets(&self) -> &mut AssetLoader {
        unsafe { &mut *self.assets }
    }
}
unsafe impl Send for Services {}
unsafe impl Sync for Services {}
