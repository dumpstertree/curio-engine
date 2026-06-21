mod callbacks;
mod commands;
pub mod game {
    pub mod capture;
    pub mod encoding;
    pub mod gpu;
    pub mod plugin_loader;
    pub mod runner;
    pub mod runner2;
}
mod state;
mod types;
mod utils;

use curio_core::ComponentState;
use curio_core::{FormsSnapshot, LedgerSnapshot, TabGroupState};
use serde::Deserialize;
use serde::Serialize;
use state::EditorState;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // state.game_thread = Some(std::thread::spawn(move || {
    //     GameRunner::new(rx, app_handle).run();
    // }));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(EditorState::default()))
        .invoke_handler(tauri::generate_handler![
            commands::press_play,
            commands::press_pause,
            commands::press_stop,
            commands::get_scene_snapshot,
            commands::get_ledger_snapshot,
            commands::get_forms,
            commands::get_tab_group_state,
            commands::get_facets,
            commands::list_dir,
            commands::read_file_bytes,
            commands::write_file_text,
            commands::move_path,
            commands::rename_path,
            commands::create_comp_file,
            commands::delete_path,
            commands::copy_file,
            commands::rebuild_manifest,
            commands::read_manifest,
            commands::create_folder
        ])
        .run(tauri::generate_context!())
        .expect("failed to run tauri app");
}

#[derive(Default, Clone, Serialize)]
pub struct SharedGameData {
    pub forms: FormsSnapshot,
    pub ledger: LedgerSnapshot,
    pub plugin: TabGroupState,
    pub facets: Box<Vec<ComponentState>>,
}

use std::sync::LazyLock;

pub static SHARED_DATA: LazyLock<Mutex<SharedGameData>> = LazyLock::new(|| Mutex::new(SharedGameData::default()));

pub static mut PROJECT: Option<Mutex<Project>> = None;

#[derive(Default, Deserialize, Clone)]
pub struct Project {
    pub name: String,
    pub project_path: String,
    pub build_args: Vec<String>,
}

unsafe impl Send for Project {}
unsafe impl Sync for Project {}
