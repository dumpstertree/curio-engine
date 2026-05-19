mod callbacks;
mod commands;
mod game;
mod state;
mod types;
mod utils;

use curio_core::{FormsSnapshot, LedgerSnapshot, TabGroupState};
use serde::Serialize;
use state::EditorState;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(EditorState::default()))
        .invoke_handler(tauri::generate_handler![commands::press_play, commands::press_pause, commands::press_stop, commands::get_scene_snapshot, commands::get_ledger_snapshot, commands::get_forms,])
        .run(tauri::generate_context!())
        .expect("failed to run tauri app");
}

#[derive(Default, Clone, Serialize)]
pub struct SharedGameData {
    pub forms: FormsSnapshot,
    pub ledger: LedgerSnapshot,
    pub plugin: TabGroupState,
}

type SharedData = Mutex<SharedGameData>;

use std::sync::LazyLock;

pub static SHARED_DATA: LazyLock<Mutex<SharedGameData>> = LazyLock::new(|| Mutex::new(SharedGameData::default()));
