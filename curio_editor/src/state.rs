//! The editor's single source of truth. This replaces three things from the
//! Tauri build at once: the Rust-side `EditorState`, the `store.ts` Zustand
//! store, and most of `commands.rs` (everything that isn't file I/O) — since
//! there's no IPC boundary anymore, "call a command" and "call a method"
//! are the same thing.

use crate::anim_viewer::AnimPreview;
use crate::glb_viewer::GlbPreview;
use crate::png_viewer::PngPreview;
use crate::prefab_state::PrefabState;
use crate::project::Project;
use crate::render_shared::RenderShared;
use crate::runner::{GameMessage, GameRunner, InputEvent, SHARED_DATA};

use curio_core::{Curio, ObjectState, PluginGroupState, Services, Severity};

use std::{
    collections::{HashSet, VecDeque},
    io::{BufRead, BufReader},
    path::Path,
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread::JoinHandle,
};

use parking_lot::Mutex;

pub type ObjectPath = Vec<usize>;

// ─────────────────────────────────────────────────────────────────────────────
// Small enums
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    Stopped,
    Playing,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileStatus {
    Idle,
    Compiling,
    Success,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopTab {
    Play,
    Asset,
    Input,
    Prefab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone)]
pub struct LogLine {
    pub level: LogLevel,
    pub message: String,
    pub time: String,
}

fn timestamp() -> String {
    // No time-formatting crate pulled in for this — good enough for the
    // console overlay; swap for `time`/`chrono` if wall-clock display
    // formatting needs to change.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs() % 86_400;
    format!("{:02}:{:02}:{:02}", secs / 3600, (secs % 3600) / 60, secs % 60)
}

// ─────────────────────────────────────────────────────────────────────────────
// Compile plumbing (was a pair of global statics in commands.rs)
// ─────────────────────────────────────────────────────────────────────────────

struct CompileHandle {
    status: Mutex<CompileStatus>,
    child: Mutex<Option<Child>>,
}

impl Default for CompileHandle {
    fn default() -> Self {
        Self {
            status: Mutex::new(CompileStatus::Idle),
            child: Mutex::new(None),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// EditorState
// ─────────────────────────────────────────────────────────────────────────────

pub struct EditorState {
    pub active_tab: TopTab,

    pub mode: EditorMode,
    pub compile_status: CompileStatus,
    pub compile_error: String,
    compile: Arc<CompileHandle>,

    pub logs: VecDeque<LogLine>,
    pub unread_logs: usize,
    pub console_open: bool,
    log_polling: bool,

    pub project: Arc<Mutex<Project>>,
    pub project_path: String,

    pub tab_group_state: Option<PluginGroupState>,
    pub selected_instance: String,
    pub active_left_tab: usize,

    pub selected_object_path: Option<ObjectPath>,
    pub expanded_nodes: HashSet<String>,

    game_tx: Option<mpsc::Sender<GameMessage>>,
    game_thread: Option<JoinHandle<()>>,
    runner_started: AtomicBool,
    render_shared: Option<RenderShared>,

    /// Persistent egui texture the live game frame gets uploaded into each
    /// repaint (see `center_panel.rs::game_texture` and
    /// `runner/capture.rs`). Kept across frames and just `.set()` in place
    /// rather than recreated, to avoid texture-handle churn.
    pub game_texture_handle: Option<eframe::egui::TextureHandle>,

    pub asset: crate::asset_state::AssetState,

    // Asset-type previews — loaded lazily when a matching file is selected
    // in the Asset tab, kept around across frames until a *different* file
    // of that type is selected (avoids re-decoding/re-uploading every
    // repaint while the same asset is just sitting there being looked at).
    pub glb_preview: Option<GlbPreview>,
    pub glb_preview_error: Option<String>,
    pub png_preview: Option<PngPreview>,
    pub png_preview_error: Option<String>,
    pub anim_preview: Option<AnimPreview>,
    pub anim_preview_error: Option<String>,

    pub prefab: PrefabState,
    pub prefab_scene: Option<crate::prefab_viewer::PrefabScene>,
}

impl EditorState {
    pub fn new(project: Project) -> Self {
        let project_path = project.project_path.clone();
        Self {
            active_tab: TopTab::Play,
            mode: EditorMode::Stopped,
            compile_status: CompileStatus::Idle,
            compile_error: String::new(),
            compile: Arc::new(CompileHandle::default()),
            logs: VecDeque::new(),
            unread_logs: 0,
            console_open: false,
            log_polling: false,
            project: Arc::new(Mutex::new(project)),
            project_path,
            tab_group_state: None,
            selected_instance: String::new(),
            active_left_tab: 0,
            selected_object_path: None,
            expanded_nodes: HashSet::new(),
            game_tx: None,
            game_thread: None,
            runner_started: AtomicBool::new(false),
            render_shared: None,
            game_texture_handle: None,
            asset: crate::asset_state::AssetState::new(),
            glb_preview: None,
            glb_preview_error: None,
            png_preview: None,
            png_preview_error: None,
            anim_preview: None,
            anim_preview_error: None,
            prefab: PrefabState::new(),
            prefab_scene: None,
        }
    }

    /// Called once from `CurioEditorApp::new`, right after construction —
    /// hands over the `Device`/`Queue`/`Renderer` eframe is using so the
    /// GLB/PNG/Spine/prefab previews can share it for zero-copy texture
    /// display. The game runner does NOT use this (see `runner/capture.rs`'s
    /// doc comment) — it has its own fully private headless device.
    pub fn set_render_shared(&mut self, render_shared: RenderShared) {
        self.render_shared = Some(render_shared);
    }

    // ── Lifecycle ────────────────────────────────────────────────────────────

    /// Spawns the game-runner thread once. Safe to call every frame.
    pub fn ensure_runner_started(&mut self) {
        if self.runner_started.swap(true, Ordering::SeqCst) {
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.game_tx = Some(tx);
        let project = self.project.clone();
        self.game_thread = Some(std::thread::spawn(move || {
            GameRunner::new(rx, project).run();
        }));
    }

    pub fn render_shared(&self) -> Option<&RenderShared> {
        self.render_shared.as_ref()
    }

    /// Call each frame the Asset tab's center panel is showing a `.png` or
    /// `.glb` preview, with the currently-selected path. No-ops (cheaply —
    /// just a string compare) if the right preview is already loaded;
    /// (re)loads from disk if the selection changed since last frame or the
    /// previous load attempt failed.
    pub fn ensure_glb_preview(&mut self, path: &str) {
        if self.glb_preview.as_ref().map(|p| p.path.as_str()) == Some(path) {
            return;
        }
        let Some(render_shared) = self.render_shared.clone() else { return };
        self.glb_preview = None;
        self.glb_preview_error = None;

        match crate::fs_ops::read_file_bytes(path) {
            Ok(bytes) => match GlbPreview::load(path.to_string(), &bytes, render_shared.device.clone(), render_shared.queue.clone()) {
                Ok(preview) => self.glb_preview = Some(preview),
                Err(e) => self.glb_preview_error = Some(e),
            },
            Err(e) => self.glb_preview_error = Some(format!("Failed to read file: {e}")),
        }
    }

    pub fn ensure_png_preview(&mut self, ctx: &eframe::egui::Context, path: &str) {
        if self.png_preview.as_ref().map(|p| p.path.as_str()) == Some(path) {
            return;
        }
        self.png_preview = None;
        self.png_preview_error = None;

        match crate::fs_ops::read_file_bytes(path) {
            Ok(bytes) => match PngPreview::load(ctx, path.to_string(), &bytes) {
                Ok(preview) => self.png_preview = Some(preview),
                Err(e) => self.png_preview_error = Some(e),
            },
            Err(e) => self.png_preview_error = Some(format!("Failed to read file: {e}")),
        }
    }

    pub fn ensure_anim_preview(&mut self, path: &str) {
        if self.anim_preview.as_ref().map(|p| p.path.as_str()) == Some(path) {
            return;
        }
        let Some(render_shared) = self.render_shared.clone() else { return };
        self.anim_preview = None;
        self.anim_preview_error = None;

        match crate::fs_ops::read_file_bytes(path) {
            Ok(bytes) => match AnimPreview::load(path.to_string(), &bytes, render_shared.device.clone(), render_shared.queue.clone()) {
                Ok(preview) => self.anim_preview = Some(preview),
                Err(e) => self.anim_preview_error = Some(e),
            },
            Err(e) => self.anim_preview_error = Some(format!("Failed to read file: {e}")),
        }
    }

    fn send(&self, msg: GameMessage) {
        if let Some(tx) = &self.game_tx {
            tx.send(msg).ok();
        }
    }

    /// Forwards a pointer/keyboard event from the viewport widget to the
    /// running game — mirrors `api.sendInput` / `ViewportCanvas`'s
    /// onPointerMove/onKeyDown handlers from the Tauri build.
    pub fn send_input(&self, event: InputEvent) {
        if self.mode != EditorMode::Playing {
            return;
        }
        self.send(GameMessage::Input(event));
    }

    // ── Playback ─────────────────────────────────────────────────────────────

    /// Kicks off a build, then (once it succeeds) starts the game runner.
    /// Mirrors the old store's `play()` — polling is now just "check this
    /// each frame" instead of a `setInterval`.
    pub fn play(&mut self) {
        self.compile_status = CompileStatus::Compiling;
        self.compile_error.clear();
        self.clear_logs();
        self.log_polling = true;
        self.spawn_compile();
    }

    pub fn stop(&mut self) {
        if *self.compile.status.lock() == CompileStatus::Compiling {
            self.cancel_compile();
        }
        self.send(GameMessage::Stop);
        self.log_polling = false;
        self.mode = EditorMode::Stopped;
        self.compile_status = CompileStatus::Idle;
        self.compile_error.clear();
        self.clear_logs();
        self.game_texture_handle = None;
        // Left panel (object tree), inspector, and the status bar's object/
        // instance counts all key off `tab_group_state` — clearing it (and
        // the selection state that pointed into it) is what makes them go
        // empty on stop instead of showing the last frame's data. Paired
        // with the `mode != Stopped` guard in `refresh_tab_group`'s caller
        // below, so it doesn't immediately get refilled with stale data
        // still sitting in `SHARED_DATA` from the run that just ended.
        self.tab_group_state = None;
        self.selected_instance.clear();
        self.active_left_tab = 0;
        self.selected_object_path = None;
        self.expanded_nodes.clear();
    }

    pub fn pause(&mut self) {
        match self.mode {
            EditorMode::Playing => {
                self.send(GameMessage::Pause);
                self.mode = EditorMode::Paused;
            }
            EditorMode::Paused => {
                self.send(GameMessage::Resume);
                self.mode = EditorMode::Playing;
            }
            EditorMode::Stopped => {}
        }
    }

    fn press_play_start(&mut self) {
        self.ensure_runner_started();
        self.send(GameMessage::Start);
        self.mode = EditorMode::Playing;
    }

    // ── Per-frame tick ───────────────────────────────────────────────────────
    // Called once per egui frame from `CurioEditorApp::update`. Replaces the
    // old `setInterval`-based pollers (compile status, tab-group refresh,
    // log polling) with plain "check on every repaint" — cheap enough here
    // since everything's a `Mutex` lock, not an IPC round-trip.

    pub fn tick(&mut self) {
        self.poll_compile();
        self.poll_logs();
        if self.mode != EditorMode::Stopped {
            self.refresh_tab_group();
        }
    }

    fn poll_compile(&mut self) {
        if self.compile_status != CompileStatus::Compiling {
            return;
        }
        let status = *self.compile.status.lock();
        match status {
            CompileStatus::Success => {
                self.press_play_start();
                self.compile_status = CompileStatus::Success;
            }
            CompileStatus::Error => {
                self.compile_status = CompileStatus::Error;
                self.compile_error = "Build failed — see console for details".to_string();
                // keep log polling running so the user can read the errors
            }
            _ => {}
        }
    }

    fn poll_logs(&mut self) {
        if !self.log_polling {
            return;
        }
        for (sev, msg) in Services::get().logger().drain() {
            let level = match sev {
                Severity::Info => "[INFO]",
                Severity::Warning => "[WARN]",
                Severity::Error => "[ERROR]",
            };
            self.push_log(format!("{level}: {msg}"));
        }
    }

    pub fn refresh_tab_group(&mut self) {
        let tab_group_state = SHARED_DATA.lock().plugin.clone();
        let keys: Vec<String> = tab_group_state.id_for_tabs.keys().cloned().collect();
        let valid_key = if keys.iter().any(|k| k == &self.selected_instance) { self.selected_instance.clone() } else { keys.first().cloned().unwrap_or_default() };
        self.selected_instance = valid_key;
        self.tab_group_state = Some(tab_group_state);
    }

    // ── Logs ─────────────────────────────────────────────────────────────────

    pub fn push_log(&mut self, line: String) {
        let lower = line.to_lowercase();
        let level = if lower.contains("[error]") || lower.contains("error:") || lower.contains("panicked") {
            LogLevel::Error
        } else if lower.contains("[warn]") || lower.contains("warning:") {
            LogLevel::Warn
        } else {
            LogLevel::Info
        };

        self.logs
            .push_back(LogLine { level, message: line, time: timestamp() });
        while self.logs.len() > 500 {
            self.logs.pop_front();
        }
        if !self.console_open {
            self.unread_logs += 1;
        }
    }

    pub fn clear_logs(&mut self) {
        self.logs.clear();
        self.unread_logs = 0;
    }

    pub fn toggle_console(&mut self) {
        self.console_open = !self.console_open;
        if self.console_open {
            self.unread_logs = 0;
        }
    }

    // ── Compile process management ──────────────────────────────────────────

    fn spawn_compile(&mut self) {
        let project = self.project.lock().clone();
        *self.compile.status.lock() = CompileStatus::Compiling;
        *self.compile.child.lock() = None;

        let handle = self.compile.clone();
        std::thread::spawn(move || {
            let mut command = Command::new("cargo");
            command.arg("build");
            for arg in &project.build_args {
                command.arg(arg);
            }
            command
                .current_dir(&project.project_path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            let mut child = match command.spawn() {
                Ok(c) => c,
                Err(e) => {
                    Curio::log(Severity::Error, &format!("Failed to spawn cargo: {e}"));
                    *handle.status.lock() = CompileStatus::Error;
                    return;
                }
            };

            if let Some(stderr) = child.stderr.take() {
                for line in BufReader::new(stderr).lines().flatten() {
                    Curio::log(Severity::Info, &line);
                }
            }

            *handle.child.lock() = Some(child);

            let status = handle.child.lock().as_mut().unwrap().wait().unwrap();
            *handle.status.lock() = if status.success() { CompileStatus::Success } else { CompileStatus::Error };
            *handle.child.lock() = None;
        });
    }

    fn cancel_compile(&mut self) {
        if let Some(mut child) = self.compile.child.lock().take() {
            child.kill().ok();
        }
        *self.compile.status.lock() = CompileStatus::Idle;
    }

    // ── Object tree selection ───────────────────────────────────────────────

    pub fn selected_object(&self) -> Option<&ObjectState> {
        let path = self.selected_object_path.as_ref()?;
        resolve_object(self.tab_group_state.as_ref()?, &self.selected_instance, self.active_left_tab, path)
    }

    pub fn select_instance(&mut self, key: String) {
        self.selected_instance = key;
        self.active_left_tab = 0;
        self.selected_object_path = None;
    }

    pub fn set_active_left_tab(&mut self, idx: usize) {
        self.active_left_tab = idx;
        self.selected_object_path = None;
    }

    pub fn select_object_by_path(&mut self, path: Option<ObjectPath>) {
        self.selected_object_path = path;
    }

    pub fn toggle_node(&mut self, path: &str) {
        if !self.expanded_nodes.remove(path) {
            self.expanded_nodes.insert(path.to_string());
        }
    }
}

fn resolve_object<'a>(tab_group_state: &'a PluginGroupState, selected_instance: &str, active_left_tab: usize, path: &[usize]) -> Option<&'a ObjectState> {
    let tabs = tab_group_state.id_for_tabs.get(selected_instance)?;
    let mut nodes: &Vec<ObjectState> = &tabs.get(active_left_tab)?.objects;
    let mut obj: Option<&ObjectState> = None;
    for &idx in path {
        obj = nodes.get(idx);
        let o = obj?;
        nodes = &o.children;
    }
    obj
}

// Convenience used by the asset/project pane later; kept here since it reads
// the same `Project`/working-dir data as everything else in this module.
pub fn project_facet_manifest_path(project_path: &str) -> std::path::PathBuf {
    Path::new(project_path).join("facet.manifest")
}
