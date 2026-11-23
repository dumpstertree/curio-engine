#[cfg(test)]
mod tests {
    use curio_engine::plugin_loader::{library_slot, load_library, lock_process_memory};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn library_slot_starts_empty() {
        // The one static dlopen slot must start as None — nothing loads a
        // plugin until load_library succeeds.
        let slot = library_slot().lock().unwrap();
        assert!(slot.is_none());
    }

    #[test]
    fn load_library_missing_source_errors() {
        // A plugin path that does not exist must surface as Err (load_library
        // copies the file into ./data first and fs::copy fails there) —
        // the runner relies on this Err to skip a stale plugin candidate.
        let dir = tempdir().unwrap();
        let missing = dir.path().join("no-such-plugin.so");

        let result = load_library(&missing);
        assert!(result.is_err());

        // load_library created ./data before failing — leave no trace behind
        let _ = fs::remove_dir_all("./data");

        // the failed load must not have left a half-loaded library behind
        let slot = library_slot().lock().unwrap();
        assert!(slot.is_none());
    }

    #[test]
    fn lock_process_memory_never_panics() {
        // mlockall may legitimately fail under sandboxing (it reports and
        // keeps going) — calling it must never take the process down.
        lock_process_memory();
    }
}
