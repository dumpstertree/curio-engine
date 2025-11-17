use super::model_asset::ModelAsset;
use super::texture_asset::TextureAsset;
use crate::collections::material::Material;
use crate::collections::material::ShaderDesc;
use crate::collections::matrix4x4::Matrix4x4;
use crate::collections::mesh::Mesh;
use crate::collections::mesh::Vertex;
use crate::io::asset_database::AssetDatabase;
use crate::io::file::File;
use crate::io::font_asset::FontAsset;
use crate::io::model_asset_animated::ModelAssetAnimated;
use core::panic;
use egui_wgpu::wgpu::Device;
use egui_wgpu::wgpu::ShaderModule;
use rusty_spine::AnimationStateData;
use rusty_spine::Atlas;
use rusty_spine::SkeletonData;
use rusty_spine::SkeletonJson;
use std::error::Error;
use std::fs;
use std::io::Cursor;
use std::sync::Arc;
use std::sync::Mutex;
use zip::ZipArchive;

static mut ASSET_DATABASE: Mutex<Option<AssetDatabase>> = Mutex::new(None);

pub struct AssetLoader {}
// private
impl AssetLoader {
    // set database
    pub fn set_database(database: AssetDatabase) {
        unsafe {
            let mut guard = ASSET_DATABASE.lock().unwrap();
            *guard = Some(database);
        }
    }

    // load - from path
    pub fn load_texture_from_path(path: &str) -> TextureAsset {
        let data = File::read(path);
        if data.len() == 0 {
            eprintln!("Something went wrong and data came back as empty for path {}", path);
            panic!();
        }

        let result = Self::unwrap_texture(&data);
        let Ok(result) = result else {
            eprintln!("Something went wrong: {}", result.err().unwrap());
            panic!();
        };

        result
    }

    // load - from database
    pub fn load_model_static_from_database(uid: String) -> Arc<ModelAsset> {
        unsafe {
            let Ok(guard) = ASSET_DATABASE.lock() else {
                panic!();
            };

            match &(*guard) {
                None => panic!("AssetDatabase has not been set"),
                Some(x) => {
                    let data = x.fetch_asset(uid);

                    if data.len() == 0 {
                        panic!("No data!");
                    }
                    let unwraped_gltf = Self::unwrap_gltf(data.as_slice()).unwrap();

                    return Arc::new(ModelAsset::new(unwraped_gltf.0, unwraped_gltf.1));
                }
            }
        }
    }
    pub fn load_model_animated_from_database(uid: String) -> Arc<ModelAssetAnimated> {
        unsafe {
            let Ok(guard) = ASSET_DATABASE.lock() else {
                panic!();
            };

            match &(*guard) {
                None => panic!("AssetDatabase has not been set"),
                Some(x) => {
                    let data = x.fetch_asset(uid);
                    let spine_data = Self::unwrap_spine(data.as_slice());
                    let Ok(spine_data) = spine_data else {
                        panic!("Err {}", spine_data.err().unwrap());
                    };

                    //create a material
                    let shader_desc = AssetLoader::load_shader_desc("assets/shader/my_shader.shader");

                    let mut material = Material::new(shader_desc.clone());
                    material.set_texture_with_label(Some(spine_data.2), "diffuse");

                    // create the asset
                    let x = Arc::new(ModelAssetAnimated::new(Arc::new(material), spine_data.1.clone(), Arc::new(AnimationStateData::new(spine_data.1.clone()))));

                    return x;
                }
            }
        }
    }

    // unwrap
    pub fn unwrap_texture(data: &[u8]) -> Result<TextureAsset, Box<dyn Error>> {
        let image = image::load_from_memory(&data).unwrap();
        let texture = TextureAsset::new_from_buffer(None, image.width(), image.height(), image.as_bytes());
        Ok(texture)
    }
    pub fn unwrap_spine(data: &[u8]) -> Result<(Arc<Atlas>, Arc<SkeletonData>, TextureAsset), Box<dyn Error>> {
        // Wrap the data so zip can read from it like a file
        let reader = Cursor::new(data);
        let mut archive = ZipArchive::new(reader)?;

        // Try to read the files
        let mut json_bytes = Vec::new();
        let mut atlas_bytes = Vec::new();
        let mut texture_bytes = Vec::new();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();

            if name.ends_with(".json") {
                std::io::copy(&mut file, &mut json_bytes)?;
            } else if name.ends_with(".atlas") {
                std::io::copy(&mut file, &mut atlas_bytes)?;
            } else if name.ends_with(".png") {
                std::io::copy(&mut file, &mut texture_bytes)?;
            }
        }

        if json_bytes.is_empty() || atlas_bytes.is_empty() || texture_bytes.is_empty() {
            return Err("Missing .json or .atlas or .png in ZIP".into());
        }

        let atlas = Arc::new(Atlas::new(atlas_bytes.as_slice(), "").unwrap());

        let mut json = SkeletonJson::new(atlas.clone());
        json.set_scale(0.01);
        let skeleton_data = Arc::new(json.read_skeleton_data(json_bytes.as_slice()).unwrap());
        let image = image::load_from_memory_with_format(&texture_bytes, image::ImageFormat::Png).unwrap();
        let texture = TextureAsset::new_from_buffer(None, image.width(), image.height(), image.as_bytes());
        Ok((atlas, skeleton_data, texture))
    }
    pub fn unwrap_gltf(data: &[u8]) -> Option<(Vec<Arc<Mesh>>, Vec<Arc<Material>>)> {
        // Import GLTF directly from memory slice
        let (gltf, buffers, images) = gltf::import_slice(data).ok()?;

        let mut all_meshes = Vec::with_capacity(gltf.meshes().len());
        let mut all_materials = Vec::with_capacity(gltf.materials().len());

        // --- Materials ---
        if gltf.materials().count() == 0 {
            let shader_desc = AssetLoader::load_shader_desc("assets/shader/unlit_shader.shader");
            all_materials.push(Arc::new(Material::new(shader_desc.clone())));
        } else {
            for material in gltf.materials() {
                println!("got material with name : {} ", material.name().unwrap());
                let pbr = material.pbr_metallic_roughness();

                let texture_asset = if let Some(tex_info) = pbr.base_color_texture() {
                    let image = &images[tex_info.texture().index()];
                    let mut pixels = image.pixels.clone();

                    // Convert 3-channel to 4-channel (R8G8B8 → R8G8B8A8)
                    if image.format == gltf::image::Format::R8G8B8 {
                        let mut rgba = Vec::with_capacity((image.width * image.height * 4) as usize);
                        for chunk in pixels.chunks(3) {
                            rgba.extend_from_slice(chunk);
                            rgba.push(255); // default alpha
                        }
                        pixels = rgba;
                    }

                    TextureAsset::new_from_buffer(None, image.width, image.height, &pixels)
                } else {
                    TextureAsset::default()
                };

                let shader_desc: ShaderDesc;
                if material.name().unwrap().starts_with("lit:") {
                    shader_desc = AssetLoader::load_shader_desc("assets/shader/my_shader.shader");
                } else if material.name().unwrap().starts_with("unlit:") {
                    shader_desc = AssetLoader::load_shader_desc("assets/shader/unlit_shader.shader");
                } else {
                    shader_desc = AssetLoader::load_shader_desc("assets/shader/my_shader.shader");
                }

                let mut mat = Material::new(shader_desc.clone());
                mat.set_texture_with_label(Some(texture_asset), "diffuse");
                all_materials.push(Arc::new(mat));
            }
        }

        // --- Meshes ---
        for mesh in gltf.meshes() {
            let mesh_name = mesh.name().unwrap_or("Unnamed");
            println!("Adding mesh: {}", mesh_name);

            for (primitive_index, primitive) in mesh.primitives().enumerate() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                // Positions are required
                let positions: Vec<[f32; 3]> = reader.read_positions()?.collect();
                let vertex_count = positions.len();
                let mut vertices = vec![Vertex::default(); vertex_count];

                // Fill position data
                for (v, pos) in vertices.iter_mut().zip(positions) {
                    v.position = pos;
                }

                // Fill optional attributes
                if let Some(normals) = reader.read_normals() {
                    for (v, normal) in vertices.iter_mut().zip(normals) {
                        v.normal = normal;
                    }
                }

                if let Some(tex0) = reader.read_tex_coords(0) {
                    for (v, uv) in vertices.iter_mut().zip(tex0.into_f32()) {
                        v.uv0 = uv;
                    }
                }

                if let Some(tex1) = reader.read_tex_coords(1) {
                    for (v, uv) in vertices.iter_mut().zip(tex1.into_f32()) {
                        v.uv1 = uv;
                    }
                }

                if let Some(colors) = reader.read_colors(0) {
                    for (v, color) in vertices.iter_mut().zip(colors.into_rgba_f32()) {
                        v.color = color;
                    }
                }

                // Indices
                let indices: Vec<u32> = reader
                    .read_indices()
                    .map(|i| i.into_u32().collect())
                    .unwrap_or_default();

                let mesh_id = format!("{}:{}", mesh_name, primitive_index);
                all_meshes.push(Arc::new(Mesh::new(mesh_id, vertices, indices, Matrix4x4::default())));
            }
        }

        Some((all_meshes, all_materials))
    }

    // load
    pub fn load_shader_module(device: &Device, path: &str) -> ShaderModule {
        let contents = fs::read_to_string(path).expect("Should have been able to read the file");
        device.create_shader_module(egui_wgpu::wgpu::ShaderModuleDescriptor {
            label: Some(path),
            source: egui_wgpu::wgpu::ShaderSource::Wgsl(contents.into()),
        })
    }
    pub fn load_shader_desc(path: &str) -> ShaderDesc {
        let file = fs::File::open(path).expect("file should open read only");
        let json: serde_json::Value = serde_json::from_reader(file).expect("file should be proper JSON");
        let my_struct: ShaderDesc = serde_json::from_str(&json.to_string()).unwrap();
        my_struct
    }
    pub fn load_font_asset_from_path(path: &str) -> FontAsset {
        let file = File::read(path);
        let json: serde_json::Value = serde_json::from_slice(file.as_slice()).expect("file should be proper JSON");
        let my_struct: FontAsset = serde_json::from_str(&json.to_string()).unwrap();
        my_struct
    }
}
