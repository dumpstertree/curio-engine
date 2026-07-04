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

use base64::Engine;
use curio_core::{ComponentState, EngineServices, GpuHandle, Logger};
use curio_core::{FormsSnapshot, LedgerSnapshot, TabGroupState};
use serde::Deserialize;
use serde::Serialize;
use state::EditorState;
use std::sync::{Arc, Mutex};

// static CACHED_SERVICES: LazyLock<EngineServices> = LazyLock::new(|| {
//     // create const values
//     let width = 1920;
//     let height = 720;

//     //
//     let instance = Instance::new(&egui_wgpu::wgpu::InstanceDescriptor {
//         backends: egui_wgpu::wgpu::Backends::all(),
//         ..Default::default()
//     });

//     // generate everything needed for rendering
//     let window = self.get_window(event_loop);
//     let surface = self.get_surface(&instance, window.clone());
//     let adapter = self.get_adapter(&instance, surface.clone());
//     let device_queue = self.get_device_queue(adapter.clone());
//     let capabilities = surface.get_capabilities(&adapter.clone());
//     let device = device_queue.0;
//     let queue = device_queue.1;
//     let format = self.get_format(&capabilities);
//     let config = self.get_config(format, width, height, &capabilities);
//     let depth_texture = Arc::new(create_depth_texture(&device, width, height));

//     // configure the sureface for rendering - make sure to call this again when changing resolution
//     surface.configure(&device, &config);

//     //
//     let capture_texture = Arc::new(device.create_texture(&TextureDescriptor {
//         label: Some("capture_texture"),

//         size: Extent3d {
//             width: CAPTURE_WIDTH,
//             height: CAPTURE_HEIGHT,
//             depth_or_array_layers: 1,
//         },

//         mip_level_count: 1,
//         sample_count: 1,
//         dimension: TextureDimension::D2,
//         format,
//         usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC | TextureUsages::COPY_DST,
//         view_formats: &[],
//     }));

//     let bytes_per_row = crate::utils::align_to(CAPTURE_WIDTH * 4, 256);

//     //
//     let readback_buffer = device.create_buffer(&BufferDescriptor {
//         label: Some("readback_buffer"),
//         size: (bytes_per_row * CAPTURE_HEIGHT) as u64,
//         usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
//         mapped_at_creation: false,
//     });

//     // save values
//     self.capture_texture = Some(capture_texture.clone());
//     self.readback_buffer = Some(readback_buffer);
//     self.surface = Some(surface.clone());
//     self.surface_format = format;

//     // save services
//     EngineServices {
//         logger: self.logger.as_mut() as *mut Logger,
//         gpu: GpuHandle {
//             device: Arc::as_ptr(&device) as *const (),
//             queue: Arc::as_ptr(&queue) as *const (),
//             config: Arc::as_ptr(&config) as *const (),
//             window: Arc::as_ptr(&window) as *const (),
//             surface: Arc::as_ptr(&surface) as *const (),
//             depth: Arc::as_ptr(&depth_texture) as *const (),
//             capture_texture: Arc::as_ptr(&capture_texture) as *const (),
//             capture_width: CAPTURE_WIDTH,
//             capture_height: CAPTURE_HEIGHT,
//         },

//         set_fullscreen,
//         set_resolution,
//         set_cursor_visible,
//     }
// });
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
            // commands::press_play,
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
            commands::create_folder,
            commands::get_project_path,
            commands::get_logs,
            commands::compile,
            commands::get_compile_status,
            commands::cancel_compile,
            commands::press_play_start,
            commands::initialize,
            commands::send_input,
            // commands::get_frame,
            commands::set_resolution,
            commands::stream_frames
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
