use curio_core::io::file::File;
use serde::Deserialize;

/// On-disk project descriptor (`test.proj` in the working directory).
/// Mirrors the original Tauri `Project` struct exactly so existing
/// `test.proj` files keep working unchanged.
#[derive(Default, Deserialize, Clone)]
pub struct Project {
    pub name: String,
    pub project_path: String,
    #[serde(default)]
    pub build_args: Vec<String>,
}

impl Project {
    /// Loads `./test.proj` relative to the current working directory.
    /// Panics on failure, matching the previous Tauri `main.rs` behavior —
    /// the editor is not useful without a project loaded.
    pub fn load_local() -> Self {
        let bytes = File::read("./test.proj");
        match serde_yaml::from_slice::<Project>(&bytes) {
            Ok(project) => {
                println!("PROJECT LOADED: {}, {}", project.name, project.project_path);
                project
            }
            Err(e) => panic!("No project found in local directory (./test.proj): {e}"),
        }
    }
}
