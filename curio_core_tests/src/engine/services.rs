#[cfg(test)]
use curio_core::{AssetCache, AssetDatabase, AssetPipeline, Gpu, Logger, Services, Severity};
#[cfg(test)]
use std::{
    ptr,
    sync::LazyLock,
};

/// Initialises the global Services singleton once for the whole test binary.
/// Engine-layer code paths (Curio::log, Ledger::write without permission,
/// synchronizer deserialization failures...) all route through Services::get(),
/// which asserts the singleton was set. Every engine test that can reach a
/// logging path must call this first. The leaked allocations only live for
/// the duration of the test run.
#[cfg(test)]
pub fn ensure_services() {
    static KEEP_SERVICES: LazyLock<bool> = LazyLock::new(|| {
        let null_void: *const () = ptr::null() as *const ();
        let services: *mut Services = Box::into_raw(Box::new(Services {
            logger: Box::into_raw(Box::new(Logger::new(Severity::Info))),
            assets: Box::into_raw(Box::new(AssetPipeline::new(AssetCache::new(10), AssetDatabase::new()))),
            gpu: Gpu {
                device: null_void,
                queue: null_void,
                capture_texture: null_void,
                capture_width: 0,
                capture_height: 0,
            },
        }));
        Services::set(services);
        true
    });
}

#[cfg(test)]
mod tests {
    use curio_core::{Curio, Services, Severity};
    use crate::engine::services::ensure_services;

    // Services::get() asserts the singleton was initialised. That assert is
    // only observable before any test has called ensure_services(), and
    // initialisation is process-global and once-only - so it cannot be
    // tested deterministically alongside the other engine tests.

    #[test]
    fn logging_reaches_shared_logger_once_initialised() {
        ensure_services();
        Curio::log(Severity::Info, "services initialised");

        // engine logging must land in the global Logger buffer
        let entries = Services::get().logger().drain();
        assert!(
            entries.iter().any(|entry| entry.1.contains("services initialised")),
            "Curio::log must buffer into the shared Logger",
        );
    }
}
