use crate::{
    game::runner2::{peek_curio, GameMessage2, GameRunner2, InputEvent},
    state::{EditorMode, EditorState},
    types::{ComponentData, DirEntry, EntityData, SceneSnapshot},
    PROJECT, SHARED_DATA,
};

use std::{
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{mpsc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use curio_core::{get_and_clear_logs, io::file::File, ComponentState, Curio, FormsSnapshot, LedgerSnapshot, Severity, TabGroupState};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

// ─────────────────────────────────────────────────────────────────────────────
// Compile state
// ─────────────────────────────────────────────────────────────────────────────

static COMPILE_STATUS: Mutex<&'static str> = Mutex::new("idle");
static COMPILE_CHILD: Mutex<Option<Child>> = Mutex::new(None);

// ─────────────────────────────────────────────────────────────────────────────
// Lifecycle
// ─────────────────────────────────────────────────────────────────────────────

/// Called once when the React app mounts. Spins up the GameRunner2 thread so
/// it is ready before any compile or play commands arrive.
#[tauri::command]
pub fn initialize(state: State<Mutex<EditorState>>, app_handle: AppHandle) -> Result<(), String> {
    let mut state = state.lock().unwrap();

    if state.game_thread.is_none() {
        let (tx, rx) = mpsc::channel();
        state.game_tx = Some(tx);
        state.game_thread = Some(std::thread::spawn(move || {
            GameRunner2::new(rx, app_handle).run();
        }));
    }

    Ok(())
}

#[tauri::command]
pub fn press_play_start(state: State<Mutex<EditorState>>, app_handle: AppHandle) -> Result<(), String> {
    unsafe {
        let Some(ref project) = PROJECT else {
            return Err("No project".into());
        };
        let mut state = state.lock().unwrap();

        if let Some(tx) = &state.game_tx {
            tx.send(GameMessage2::Start).ok();
        }

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

    state.mode = EditorMode::Stopped;
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Input forwarding — React viewport sends pointer/keyboard events here
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn send_input(state: State<Mutex<EditorState>>, event: InputEvent) -> Result<(), String> {
    let state = state.lock().unwrap();
    if let Some(tx) = &state.game_tx {
        tx.send(GameMessage2::Input(event)).ok();
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Scene / state queries
// ─────────────────────────────────────────────────────────────────────────────

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

// ─────────────────────────────────────────────────────────────────────────────
// Facets
// ─────────────────────────────────────────────────────────────────────────────

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
    Asset(String),
    Float,
    Int,
    Bool,
    Vector2,
    Vector3,
    Vector4,
}

#[tauri::command]
pub fn get_facets(state: State<Mutex<EditorState>>) -> FacetManifest {
    const PATH: &str = "/home/dumpstertree/Git/Rust/system_test/facet.manifest";
    serde_yaml::from_slice::<FacetManifest>(&File::read(PATH))
        .ok()
        .unwrap()
}

// ─────────────────────────────────────────────────────────────────────────────
// Resolution
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn set_resolution(state: State<Mutex<EditorState>>, app_handle: AppHandle, w: u32, h: u32) {
    let state = state.lock().unwrap();
    if let Some(tx) = &state.game_tx {
        tx.send(GameMessage2::Resize(w, h)).ok();
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// File system
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn create_folder(path: String) -> Result<(), String> {
    std::fs::create_dir_all(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_dir(path: String) -> Result<Vec<DirEntry>, String> {
    let mut entries = Vec::new();
    let read = std::fs::read_dir(&path).map_err(|e| e.to_string())?;

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
    let p = Path::new(&path);
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
    if std::fs::rename(&src, &dst).is_ok() {
        return Ok(());
    }
    let sp = Path::new(&src);
    if sp.is_dir() {
        copy_dir_all(sp, Path::new(&dst))?;
        std::fs::remove_dir_all(sp).map_err(|e| e.to_string())
    } else {
        std::fs::copy(&src, &dst).map_err(|e| e.to_string())?;
        std::fs::remove_file(&src).map_err(|e| e.to_string())
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), String> {
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

// ─────────────────────────────────────────────────────────────────────────────
// Asset manifest
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn rebuild_manifest() -> Result<(), String> {
    const PROJECT_ROOT: &str = "/home/dumpstertree/Git/Rust/system_test";
    const ASSETS_ROOT: &str = "/home/dumpstertree/Git/Rust/system_test/assets";

    #[derive(Serialize)]
    struct ManifestEntry {
        id: i16,
        name: String,
        r#type: String,
        uri: String,
    }

    fn collect(dir: &Path, root: &Path, entries: &mut Vec<ManifestEntry>) -> Result<(), String> {
        for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if name.ends_with(".meta") {
                continue;
            }
            if path.is_dir() {
                collect(&path, root, entries)?;
                continue;
            }

            let meta_path = PathBuf::from(format!("{}.meta", path.to_string_lossy()));
            let meta_text = match std::fs::read_to_string(&meta_path) {
                Ok(t) => t,
                Err(_) => continue,
            };

            let mut id: Option<i16> = None;
            let mut included = true;
            for line in meta_text.lines() {
                if let Some(v) = line.strip_prefix("id:") {
                    id = v.trim().parse::<i16>().ok();
                }
                if let Some(v) = line.strip_prefix("included:") {
                    included = v.trim() != "false";
                }
            }
            let Some(id) = id else { continue };
            if !included {
                continue;
            }

            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(&name)
                .to_string();

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

    let mut yaml = String::from("manifest:\n");
    for e in &entries {
        yaml.push_str(&format!("  - id: {}\n    name: \"{}\"\n    type: {}\n    uri: \"{}\"\n", e.id, e.name, e.r#type, e.uri));
    }

    std::fs::write(format!("{}/asset.manifest", PROJECT_ROOT), yaml).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn read_manifest() -> Result<String, String> {
    let path = "/home/dumpstertree/Git/Rust/system_test/asset.manifest";
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// Project
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_project_path() -> String {
    unsafe {
        PROJECT
            .as_ref()
            .and_then(|p| p.lock().ok())
            .map(|p| p.project_path.clone())
            .unwrap_or_default()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Compilation
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn compile(state: State<Mutex<EditorState>>) -> Result<(), String> {
    unsafe {
        let Some(ref project) = PROJECT else {
            return Err("No project".into());
        };
        let p = project.lock().unwrap().clone();

        *COMPILE_STATUS.lock().unwrap() = "compiling";
        *COMPILE_CHILD.lock().unwrap() = None;

        std::thread::spawn(move || {
            let mut command = Command::new("cargo");
            command.arg("build");
            for arg in &p.build_args {
                command.arg(arg);
            }
            command
                .current_dir(&p.project_path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            let mut child = match command.spawn() {
                Ok(c) => c,
                Err(e) => {
                    Curio::log(Severity::Error, &format!("Failed to spawn cargo: {}", e));
                    *COMPILE_STATUS.lock().unwrap() = "error";
                    return;
                }
            };

            if let Some(stderr) = child.stderr.take() {
                for line in BufReader::new(stderr).lines().flatten() {
                    Curio::log(Severity::Info, &line);
                }
            }

            *COMPILE_CHILD.lock().unwrap() = Some(child);

            let status = COMPILE_CHILD
                .lock()
                .unwrap()
                .as_mut()
                .unwrap()
                .wait()
                .unwrap();

            *COMPILE_STATUS.lock().unwrap() = if status.success() { "success" } else { "error" };
            *COMPILE_CHILD.lock().unwrap() = None;
        });
    }
    Ok(())
}

#[tauri::command]
pub fn get_compile_status() -> String {
    COMPILE_STATUS.lock().unwrap().to_string()
}

#[tauri::command]
pub fn cancel_compile() -> Result<(), String> {
    if let Some(mut child) = COMPILE_CHILD.lock().unwrap().take() {
        child.kill().ok();
    }
    *COMPILE_STATUS.lock().unwrap() = "idle";
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Logs
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_logs() -> Vec<(String, String)> {
    get_and_clear_logs()
        .into_iter()
        .map(|(sev, msg)| {
            let level = match sev {
                Severity::Info => "[INFO]",
                Severity::Warning => "[WARN]",
                Severity::Error => "[ERROR]",
            }
            .to_string();
            (level, msg)
        })
        .collect()
}
