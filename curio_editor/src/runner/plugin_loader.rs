//! Dynamic loading of compiled `curio_core` game plugins (`.so`/`.dll`/`.dylib`).
//! Direct port — no Tauri dependency here in the first place.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

use libloading::Library;

static LOADED_LIBRARY: OnceLock<Mutex<Option<Library>>> = OnceLock::new();

pub fn library_slot() -> &'static Mutex<Option<Library>> {
    LOADED_LIBRARY.get_or_init(|| Mutex::new(None))
}

pub fn load_library(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut slot = library_slot().lock().unwrap();

    // Unload previous library.
    if let Some(old_lib) = slot.take() {
        drop(old_lib);
    }

    // Create data directory.
    let data_dir = PathBuf::from("./data");
    fs::create_dir_all(&data_dir)?;

    // Remove previously copied plugin files.
    for entry in fs::read_dir(&data_dir)? {
        let entry = entry?;
        let file_name = entry.file_name();

        if let Some(name) = file_name.to_str() {
            if name.starts_with("app_") {
                let _ = fs::remove_file(entry.path());
            }
        }
    }

    // Generate unique filename.
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("so");
    let copied_path = data_dir.join(format!("app_{}.{}", timestamp, extension));

    // Copy freshly-built plugin.
    fs::copy(path, &copied_path)?;

    // Load copied plugin.
    #[cfg(unix)]
    {
        use libloading::os::unix::{Library as UnixLibrary, RTLD_NOW};

        unsafe {
            let lib = UnixLibrary::open(Some(&copied_path), RTLD_NOW)?;
            *slot = Some(lib.into());
        }
    }

    #[cfg(windows)]
    {
        unsafe {
            *slot = Some(Library::new(&copied_path)?);
        }
    }

    println!("Loaded plugin: {}", copied_path.display());

    Ok(())
}
