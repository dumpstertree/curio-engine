use crate::{
    game::runner2::{GameMessage2, GameRunner2},
    state::{EditorMode, EditorState},
    types::{ComponentData, DirEntry, EntityData, SceneSnapshot},
    PROJECT, SHARED_DATA,
};

use std::{
    process::Command,
    sync::{mpsc, Mutex},
};

use curio_core::{engine_services::services, FormsSnapshot, LedgerSnapshot, LoadedCurio, TabGroupState};
use tauri::{AppHandle, State};

#[tauri::command]
pub fn press_play(state: State<Mutex<EditorState>>, app_handle: AppHandle) -> Result<(), String> {
    unsafe {
        let Some(ref project) = PROJECT else {
            panic!("No Project Loaded");
        };

        let p = project.lock().unwrap().clone();

        let mut state = state.lock().unwrap();

        match state.mode {
            EditorMode::Playing => return Ok(()),

            EditorMode::Paused => {
                if let Some(tx) = &state.game_tx {
                    tx.send(GameMessage2::Resume).ok();
                }

                state.mode = EditorMode::Playing;
                return Ok(());
            }

            EditorMode::Stopped => {}
        }
        println!("building");

        let mut command = Command::new("cargo");
        command.arg("build");
        for arg in p.build_args {
            command.arg(arg);
        }

        let status = command.current_dir(p.project_path.clone()).status();

        println!("built");

        if state.game_thread.is_none() {
            let (tx, rx) = mpsc::channel();
            state.game_tx = Some(tx);
            state.game_thread = Some(std::thread::spawn(move || {
                GameRunner2::new(rx, app_handle).run();
            }));
        }

        if let Some(x) = &state.game_tx {
            let _ = x.send(GameMessage2::Start);
        }

        state.mode = EditorMode::Playing;

        Ok(())
    }
}

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn stage_plugin(src: &Path) -> Result<PathBuf, String> {
    // copy libgame.so → libgame_1234567890.so
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let stem = src
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("libgame");

    let ext = src.extension().and_then(|s| s.to_str()).unwrap_or("so");

    let staged = src.with_file_name(format!("{}_{}.{}", stem, ts, ext));
    std::fs::copy(src, &staged).map_err(|e| e.to_string())?;
    Ok(staged)
}
#[tauri::command]
pub fn press_pause(state: State<Mutex<EditorState>>) -> Result<(), String> {
    let mut state = state.lock().unwrap();

    match state.mode {
        EditorMode::Playing => {
            if let Some(tx) = &state.game_tx {
                tx.send(GameMessage2::Pause).ok();
            }

            state.mode = EditorMode::Paused;
        }

        EditorMode::Paused => {
            if let Some(tx) = &state.game_tx {
                tx.send(GameMessage2::Resume).ok();
            }

            state.mode = EditorMode::Playing;
        }

        EditorMode::Stopped => {}
    }

    Ok(())
}

#[tauri::command]
pub fn press_stop(state: State<Mutex<EditorState>>) -> Result<(), String> {
    let mut state = state.lock().unwrap();

    if let Some(tx) = &state.game_tx {
        tx.send(GameMessage2::Stop).ok();
    }

    // state.game_tx = None;
    // state.game_thread = None;
    state.mode = EditorMode::Stopped;

    Ok(())
}

#[tauri::command]
pub fn get_scene_snapshot() -> SceneSnapshot {
    SceneSnapshot {
        entities: vec![
            EntityData {
                id: 0,
                name: "name 0".to_owned(),
                children: vec![],
                components: vec![ComponentData { name: "Test".to_owned(), fields: "A B C".into() }],
            },
            EntityData {
                id: 1,
                name: "name 1".to_owned(),
                children: vec![],
                components: vec![],
            },
            EntityData {
                id: 2,
                name: "name 2".to_owned(),
                children: vec![],
                components: vec![],
            },
            EntityData {
                id: 3,
                name: "name 3".to_owned(),
                children: vec![],
                components: vec![],
            },
        ],
    }
}

#[tauri::command]
pub fn get_ledger_snapshot(state: State<Mutex<EditorState>>) -> LedgerSnapshot {
    let Ok(data) = SHARED_DATA.lock() else {
        panic!("Failed to get data");
    };

    data.ledger.clone()
}

#[tauri::command]
pub fn get_forms(state: State<Mutex<EditorState>>) -> FormsSnapshot {
    let Ok(data) = SHARED_DATA.lock() else {
        panic!("Failed to get data");
    };

    data.forms.clone()
}

#[tauri::command]
pub fn get_tab_group_state(state: State<Mutex<EditorState>>) -> TabGroupState {
    let Ok(data) = SHARED_DATA.lock() else {
        panic!("Failed to get data");
    };

    data.plugin.clone()
}

#[tauri::command]
pub fn set_resolution(state: State<Mutex<EditorState>>, app_handle: AppHandle, w: u32, h: u32) {
    let state = state.lock().unwrap();
    if let Some(tx) = &state.game_tx {
        tx.send(GameMessage2::Resize(w, h)).ok();
    }
}
#[tauri::command]
pub fn list_dir(path: String) -> Result<Vec<DirEntry>, String> {
    use std::fs;

    let mut entries = Vec::new();
    let read = fs::read_dir(&path).map_err(|e| e.to_string())?;

    for entry in read.flatten() {
        let metadata = entry.metadata().map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        let full_path = entry.path().to_string_lossy().to_string();
        entries.push(DirEntry { name, path: full_path, is_dir: metadata.is_dir() });
    }

    entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));

    Ok(entries)
}

#[tauri::command]
pub fn read_file_bytes(path: String) -> Result<Vec<u8>, String> {
    std::fs::read(&path).map_err(|e| e.to_string())
}
