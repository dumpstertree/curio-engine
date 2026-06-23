use crate::{
    game::runner2::{peek_curio, GameMessage2, GameRunner2},
    state::{EditorMode, EditorState},
    types::{ComponentData, DirEntry, EntityData, SceneSnapshot},
    PROJECT, SHARED_DATA,
};

use std::{
    process::Command,
    sync::{mpsc, Mutex},
};

use curio_core::{io::file::File, ComponentState, FormsSnapshot, LedgerSnapshot, TabGroupState};
use serde::{Deserialize, Serialize};
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
pub fn get_facets(state: State<Mutex<EditorState>>) -> FacetManifest {
    const PATH: &str = "/home/dumpstertree/git/rust/curio-engine-demo/facet.manifest";

    let x = serde_yaml::from_slice::<FacetManifest>(&File::read(PATH))
        .ok()
        .unwrap();
    x
}

#[tauri::command]
pub fn create_folder(path: String) -> Result<(), String> {
    std::fs::create_dir_all(&path).map_err(|e| e.to_string())
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
#[tauri::command]
pub fn write_file_text(path: String, contents: String) -> Result<(), String> {
    std::fs::write(&path, contents).map_err(|e| e.to_string())
}
#[tauri::command]
pub fn copy_file(src: String, dst: String) -> Result<(), String> {
    std::fs::copy(&src, &dst)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_comp_file(path: String) -> Result<(), String> {
    let contents = "enabled: true\nname: \"New GameObject\"\ncomponents: []\nchildren: []\n";
    std::fs::write(&path, contents).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_path(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    if p.is_dir() {
        std::fs::remove_dir_all(p).map_err(|e| e.to_string())
    } else {
        std::fs::remove_file(p).map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub fn rename_path(old_path: String, new_path: String) -> Result<(), String> {
    std::fs::rename(&old_path, &new_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn move_path(src: String, dst: String) -> Result<(), String> {
    // Try rename first (same filesystem), fall back to copy+delete
    if std::fs::rename(&src, &dst).is_ok() {
        return Ok(());
    }
    let sp = std::path::Path::new(&src);
    if sp.is_dir() {
        copy_dir_all(sp, std::path::Path::new(&dst))?;
        std::fs::remove_dir_all(sp).map_err(|e| e.to_string())
    } else {
        std::fs::copy(&src, &dst).map_err(|e| e.to_string())?;
        std::fs::remove_file(&src).map_err(|e| e.to_string())
    }
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    for entry in std::fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let ty = entry.file_type().map_err(|e| e.to_string())?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name())).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
#[tauri::command]
pub fn rebuild_manifest() -> Result<(), String> {
    use std::fs;
    use std::path::{Path, PathBuf};

    const PROJECT_ROOT: &str = "/home/dumpstertree/git/rust/curio-engine-demo";
    const ASSETS_ROOT: &str = "/home/dumpstertree/git/rust/curio-engine-demo/assets";
    #[derive(serde::Serialize)]
    struct ManifestEntry {
        id: i16, // ← was u64
        name: String,
        r#type: String,
        uri: String,
    }
    fn collect(dir: &Path, root: &Path, entries: &mut Vec<ManifestEntry>) -> Result<(), String> {
        for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip .meta files and directories (folders don't get manifest entries)
            if name.ends_with(".meta") {
                continue;
            }
            if path.is_dir() {
                collect(&path, root, entries)?;
                continue;
            }

            // Read .meta file for id and included flag
            let meta_path = PathBuf::from(format!("{}.meta", path.to_string_lossy()));
            let meta_text = match fs::read_to_string(&meta_path) {
                Ok(t) => t,
                Err(_) => continue, // no meta = not registered, skip
            };

            // Parse included and id from YAML (simple line scan to avoid serde_yaml dep)
            let mut id: Option<i16> = None;
            let mut included: bool = true;
            for line in meta_text.lines() {
                if let Some(v) = line.strip_prefix("id:") {
                    id = v.trim().parse::<i16>().ok();
                }
                if let Some(v) = line.strip_prefix("included:") {
                    included = v.trim() != "false";
                }
            }
            let Some(id): Option<i16> = id else { continue };
            if !included {
                continue;
            }

            // name = filename without extension(s)
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(&name)
                .to_string();

            // uri = path relative to project root
            let uri = path
                .strip_prefix(root)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| name.clone());

            entries.push(ManifestEntry { id, name: stem, r#type: "Embedded".to_string(), uri });
        }
        Ok(())
    }

    let mut entries = Vec::new();
    collect(Path::new(ASSETS_ROOT), Path::new(PROJECT_ROOT), &mut entries)?;

    // Build YAML manually for clean formatting
    let mut yaml = String::from("manifest:\n");
    for e in &entries {
        yaml.push_str(&format!("  - id: {}\n    name: \"{}\"\n    type: {}\n    uri: \"{}\"\n", e.id, e.name, e.r#type, e.uri));
    }

    let manifest_path = format!("{}/asset.manifest", PROJECT_ROOT);
    fs::write(&manifest_path, yaml).map_err(|e| e.to_string())
}
#[tauri::command]
pub fn read_manifest() -> Result<String, String> {
    let path = "/home/dumpstertree/git/rust/curio-engine-demo/asset.manifest";
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

#[derive(Serialize, Deserialize)]
pub struct FacetManifest {
    manifest: Vec<FacetManifestEntry>,
}
#[derive(Serialize, Deserialize)]
pub struct FacetManifestEntry {
    name: String,
    data: Vec<FacetManifestEntryField>,
}
#[derive(Serialize, Deserialize)]
pub struct FacetManifestEntryField {
    name: String,
    data: EntryTypes,
}
#[derive(Serialize, Deserialize)]
pub enum EntryTypes {
    Asset(String), // where string is suffix
    Float,
    Int,
    Bool,
    Vector2,
    Vector3,
    Vector4,
}
