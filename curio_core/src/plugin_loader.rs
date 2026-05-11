// engine/src/plugin_loader.rs

use std::path::Path;

use libloading::Library;

pub fn load_library(path: &Path) -> Result<Library, libloading::Error> {
    #[cfg(unix)]
    {
        use libloading::os::unix::{Library as UnixLibrary, RTLD_GLOBAL, RTLD_NOW};
        unsafe {
            let lib = UnixLibrary::open(Some(path), RTLD_NOW | RTLD_GLOBAL)?;
            Ok(lib.into())
        }
    }
    #[cfg(windows)]
    {
        // windows doesn't have this problem — PE format handles
        // symbol resolution differently, normal load works
        unsafe { Library::new(path) }
    }
}
