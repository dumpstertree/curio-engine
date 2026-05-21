use crate::{
    game::{GameMessage, GameRunner},
    state::{EditorMode, EditorState},
    types::{ComponentData, EntityData, SceneSnapshot},
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
                    tx.send(GameMessage::Resume).ok();
                }

                state.mode = EditorMode::Playing;
                return Ok(());
            }

            EditorMode::Stopped => {}
        }

        let (tx, rx) = mpsc::channel();

        state.game_tx = Some(tx);

        let mut command = Command::new("cargo");
        command.arg("build");
        for arg in p.build_args {
            command.arg(arg);
        }

        println!("building");
        // state.game_thread = Some(std::thread::spawn(move || {
        let status = command.current_dir(p.project_path.clone()).status();

        let Ok(stat) = status else { panic!() };
        // }));

        println!("built");

        state.game_thread = Some(std::thread::spawn(move || {
            GameRunner::new(rx, app_handle).run();
        }));

        state.mode = EditorMode::Playing;

        Ok(())
    }
}

#[tauri::command]
pub fn press_pause(state: State<Mutex<EditorState>>) -> Result<(), String> {
    let mut state = state.lock().unwrap();

    match state.mode {
        EditorMode::Playing => {
            if let Some(tx) = &state.game_tx {
                tx.send(GameMessage::Pause).ok();
            }

            state.mode = EditorMode::Paused;
        }

        EditorMode::Paused => {
            if let Some(tx) = &state.game_tx {
                tx.send(GameMessage::Resume).ok();
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
        tx.send(GameMessage::Stop).ok();
    }

    state.game_tx = None;
    state.game_thread = None;
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
        tx.send(GameMessage::Resize(w, h)).ok();
    }
}
