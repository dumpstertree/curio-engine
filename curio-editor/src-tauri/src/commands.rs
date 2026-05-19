use crate::{
    game::{GameMessage, GameRunner},
    state::{EditorMode, EditorState},
    types::{ComponentData, EntityData, SceneSnapshot},
    SHARED_DATA,
};

use std::sync::{mpsc, Mutex};

use curio_core::{FormsSnapshot, LedgerSnapshot, LoadedCurio, TabGroupState};
use tauri::{AppHandle, State};

#[tauri::command]
pub fn press_play(state: State<Mutex<EditorState>>, app_handle: AppHandle) -> Result<(), String> {
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

    state.game_thread = Some(std::thread::spawn(move || {
        GameRunner::new(rx, app_handle).run();
    }));

    state.mode = EditorMode::Playing;

    Ok(())
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

pub fn get_plugin_state() -> TabGroupState {
    let Ok(data) = SHARED_DATA.lock() else {
        panic!("Failed to get data");
    };

    data.plugin.clone()
}
