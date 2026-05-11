// engine/src/plugin.rs — this lives in your engine crate

/// The raw function signature every plugin .so must export.
/// C ABI so the linker can find it regardless of Rust version.
pub type PluginCreateFn = unsafe extern "C" fn() -> *mut dyn Plugin;

/// The trait all plugins implement — stays on the engine side
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn build(&self);
}
