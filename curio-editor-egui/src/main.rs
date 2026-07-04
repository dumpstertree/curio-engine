// Prevents an additional console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod anim_viewer;
mod app;
mod asset_state;
mod fs_ops;
mod glb_viewer;
mod panels;
mod png_viewer;
mod prefab_facets;
mod prefab_resolver;
mod prefab_state;
mod prefab_transforms;
mod prefab_types;
mod prefab_viewer;
mod project;
mod render_shared;
mod runner;
mod state;
mod theme;

use app::CurioEditorApp;
use project::Project;

fn main() -> eframe::Result<()> {
    // Load ./test.proj relative to the working directory — same convention
    // as the Tauri build's main.rs.
    let project = Project::load_local();

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1400.0, 900.0]).with_min_inner_size([900.0, 600.0]).with_title("Curio Editor"),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native("curio-editor", native_options, Box::new(|cc| Ok(Box::new(CurioEditorApp::new(cc, project)))))
}
