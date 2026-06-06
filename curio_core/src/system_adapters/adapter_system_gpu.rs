use egui_wgpu::wgpu::ShaderModule;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{engine_services::services, io::file::File, shaders::Shaders};

pub static SHADERS_MODULES: Mutex<Option<HashMap<i16, Arc<ShaderModule>>>> = Mutex::new(None);

// pub fn get_shader_module(id: &i16) -> Arc<ShaderModule> {
//     // Lock shader storage
//     let mut guard = SHADERS_MODULES
//         .lock()
//         .expect("FAILED TO LOCK SHADER MODULES");

//     // Initialize map if needed
//     let shaders = guard.get_or_insert_with(HashMap::new);

//     // Return cached if exists
//     if let Some(existing) = shaders.get(id) {
//         return existing.clone();
//     }

//     // Determine file path
//     let p1 = File::get_built_in_asset_path();
//     let mut p2 = "";
//     if let Some(cached) = shaders.get(id) {
//         return cached.clone();
//     } else {
//         if *id == ASSET_UID_SHADER_MODULE_LIT {
//             return Arc::new(Shaders::lit());
//             // p2 = "built_in/shader_module/lit.wgsl";
//         }
//         if *id == ASSET_UID_SHADER_MODULE_UNLIT {
//             return Arc::new(Shaders::unlit());
//             // p2 = "built_in/shader_module/unlit.wgsl";
//         }
//     }
//     // Read shader file
//     let data = File::read(&format!("{}/{}", p1, p2));
//     let string = String::from_utf8(data).expect("Invalid UTF8 in shader");

//     // IMPORTANT:
//     // Drop lock before creating shader (not strictly required, but cleaner and avoids deadlock risk)
//     drop(guard);

//     // Create shader module
//     let module = services()
//         .gpu
//         .device()
//         .create_shader_module(egui_wgpu::wgpu::ShaderModuleDescriptor {
//             label: Some("Shader"),
//             source: egui_wgpu::wgpu::ShaderSource::Wgsl(string.into()),
//         });

//     let asset = Arc::new(module);

//     // Re-lock and insert
//     let mut guard = SHADERS_MODULES
//         .lock()
//         .expect("FAILED TO LOCK SHADER MODULES");

//     let shaders = guard.get_or_insert_with(HashMap::new);
//     shaders.insert(*id, asset.clone());

//     asset
// }
