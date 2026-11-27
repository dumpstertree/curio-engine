#[cfg(test)]
mod tests {
    use curio_core::{Portal, Services};
    use curio_engine::curio_cabinet::{expected_plugin_stem, load_curio, CabinetWindowOwner};
    use curio_engine::project::Project;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::{fs, ptr};
    use tempfile::tempdir;

    // load_library copies plugin candidates into ./data even when it then
    // fails — every test that reaches it sweeps those copies clean.
    fn sweep_data_dir() {
        let Ok(entries) = fs::read_dir("./data") else {
            return;
        };
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let Some(name) = file_name.to_str() else {
                continue;
            };
            if name.starts_with("app_") {
                let _ = fs::remove_file(entry.path());
            }
        }
        let _ = fs::remove_dir_all("./data");
    }

    // ── plugin stem preference ───────────────────────────────────────────────

    #[test]
    fn expected_plugin_stem_prefers_package_name() {
        let dir = tempdir().unwrap();
        let manifest = "[package]\nname = \"my-game\"\nversion = \"0.1.0\"\n";
        assert!(fs::write(dir.path().join("Cargo.toml"), manifest.as_bytes()).is_ok());

        let stem = expected_plugin_stem(dir.path().to_str().unwrap());
        let Some(stem_str) = stem else {
            panic!("expected a stem for [package] crate 'my-game'");
        };
        assert_eq!(stem_str, "libmy_game");
    }

    #[test]
    fn expected_plugin_stem_ignores_names_outside_package_section() {
        // A `name` under another section (or with no [package] at all) must
        // not produce a stem — it would let a dependency's crate name steer
        // the plugin search.
        let dir = tempdir().unwrap();
        let manifest = "[dependencies]\ncurio_core = { path = \"../curio_core\" }\nname = \"decoy\"\n";
        assert!(fs::write(dir.path().join("Cargo.toml"), manifest.as_bytes()).is_ok());

        let stem = expected_plugin_stem(dir.path().to_str().unwrap());
        assert!(stem.is_none());
    }

    #[test]
    fn expected_plugin_stem_missing_manifest_is_none() {
        // No Cargo.toml at the project path (renamed/misconfigured project) —
        // None just means "no preference", the load still tries every .so.
        let dir = tempdir().unwrap();
        let stem = expected_plugin_stem(dir.path().to_str().unwrap());
        assert!(stem.is_none());
    }

    #[test]
    fn expected_plugin_stem_package_without_name_is_none() {
        let dir = tempdir().unwrap();
        let manifest = "[package]\nversion = \"0.1.0\"\n";
        assert!(fs::write(dir.path().join("Cargo.toml"), manifest.as_bytes()).is_ok());

        let stem = expected_plugin_stem(dir.path().to_str().unwrap());
        assert!(stem.is_none());
    }

    // ── plugin load failures ─────────────────────────────────────────────────
    // A successful load needs a real dlopen'd curio_init — not runnable
    // headlessly here, so these cover the failure contracts instead.

    #[test]
    fn load_curio_panics_when_plugins_folder_is_missing() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("target").join("release");

        let gpu: *const Services = ptr::null();
        let survived = catch_unwind(AssertUnwindSafe(|| load_curio(&missing, gpu, None)));
        assert!(survived.is_err());
    }

    #[test]
    fn load_curio_panics_when_folder_has_no_plugins() {
        // An empty folder yields no candidates at all.
        let dir = tempdir().unwrap();
        let folder = dir.path().join("release");
        assert!(fs::create_dir_all(&folder).is_ok());

        let gpu: *const Services = ptr::null();
        let survived = catch_unwind(AssertUnwindSafe(|| load_curio(&folder, gpu, None)));
        assert!(survived.is_err());

        sweep_data_dir();
    }

    #[test]
    fn load_curio_skips_libraries_without_curio_init() {
        // A file that exists but is not a loadable shared object: load_library
        // fails at dlopen, the candidate is skipped, and with nothing else
        // found the runner still ends at the "No curio plugin found" panic.
        let dir = tempdir().unwrap();
        let folder = dir.path().join("release");
        assert!(fs::create_dir_all(&folder).is_ok());
        assert!(fs::write(folder.join("not-a-plugin.so"), b"not an ELF at all").is_ok());

        let gpu: *const Services = ptr::null();
        let survived = catch_unwind(AssertUnwindSafe(|| load_curio(&folder, gpu, None)));
        assert!(survived.is_err());

        sweep_data_dir();
    }

    // ── per-window owner construction ────────────────────────────────────────

    #[test]
    fn cabinet_window_owner_constructs_cheaply_before_running() {
        // new() only stages Logger/AssetPipeline (no GPU, no window) — the
        // heavy work is deferred to run().
        let owner = CabinetWindowOwner::new(Portal::windowed_720(), Project::default());
        let _unused = owner;
    }
}
