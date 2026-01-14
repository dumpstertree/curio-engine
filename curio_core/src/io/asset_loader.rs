use super::model_asset::ModelAsset;
use super::texture_asset::TextureAsset;
use crate::collections::material::Material;
use crate::collections::material::ShaderDesc;
use crate::collections::matrix4x4::Matrix4x4;
use crate::collections::mesh::Mesh;
use crate::collections::mesh::Vertex;
use crate::io::asset_cache;
use crate::io::asset_cache::AssetCache;
use crate::io::asset_database;
use crate::io::asset_database::AssetDatabase;
use crate::io::asset_database::AssetDatabaseListing;
use crate::io::asset_loader;
use crate::io::file::File;
use crate::io::font_asset::FontDesc;
use crate::io::model_asset_animated::ModelAssetAnimated;
use core::panic;
use egui::cache;
use egui_wgpu::wgpu::Device;
use egui_wgpu::wgpu::ShaderModule;
use gltf::json::asset;
use rusty_spine::AnimationStateData;
use rusty_spine::Atlas;
use rusty_spine::SkeletonData;
use rusty_spine::SkeletonJson;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::io::Cursor;
use std::sync::Arc;
use std::sync::Mutex;
use zip::ZipArchive;

static mut ASSET_DATABASE: Option<Mutex<AssetDatabase>> = None;
static mut ASSET_CACHE: Option<Mutex<AssetCache>> = None;

// Built in Shaders
pub static ASSET_UID_SHADER_UNLIT: i16 = -100;
pub static ASSET_UID_SHADER_LIT: i16 = -101;

// Built in Shaders
pub static ASSET_UID_SHADER_MODULE_UNLIT: i16 = -200;
pub static ASSET_UID_SHADER_MODULE_LIT: i16 = -201;

// Built in Textures
pub static ASSET_UID_TEXTURE_FONT_ATLAS: i16 = -300;

// Font Asset
pub static ASSET_UID_FONT_ASSET_DEFAULT: i16 = -400;
pub struct AssetLoader {}
// private
impl AssetLoader {
    /// Try to find the key based on the name.
    pub fn try_lookup_key_for_name(name: &str) -> Option<i16> {
        unsafe {
            let Some(asset_database) = &ASSET_DATABASE else {
                panic!();
            };
            let Ok(asset_database) = asset_database.lock() else {
                panic!();
            };

            asset_database.try_lookup_key_for_name(name)
        }
    }
    pub fn preload_remote_assets(force: bool) {
        unsafe {
            let Some(asset_database) = &ASSET_DATABASE else {
                panic!();
            };
            let Ok(mut asset_database) = asset_database.lock() else {
                panic!();
            };

            // preload
            asset_database.preload_remote_assets(force);
        }
    }
    // set database
    pub fn set_database(database: AssetDatabase) {
        let mut database = database;
        database.append(vec![
            // shaders
            ("shader_lit".to_string(), ASSET_UID_SHADER_LIT, AssetDatabaseListing::Local("shader/my_shader.shader".to_string())),
            ("shader_unlit".to_string(), ASSET_UID_SHADER_UNLIT, AssetDatabaseListing::Local("shader/unlit_shader.shader".to_string())),
            // shader modules
            ("shader_module_lit".to_string(), ASSET_UID_SHADER_MODULE_LIT, AssetDatabaseListing::Local("shader/shader.wgsl".to_string())),
            ("shader_module_unlit".to_string(), ASSET_UID_SHADER_MODULE_UNLIT, AssetDatabaseListing::Local("shader/shader_unlit.wgsl".to_string())),
            // textures
            ("default_texture_font_atlas".to_string(), ASSET_UID_TEXTURE_FONT_ATLAS, AssetDatabaseListing::Local("font.png".to_string())),
            // font
            ("default_font_asset".to_string(), ASSET_UID_FONT_ASSET_DEFAULT, AssetDatabaseListing::Local("default.font".to_string())),
        ]);
        unsafe {
            ASSET_DATABASE = Some(Mutex::new(database));
            ASSET_CACHE = Some(Mutex::new(AssetCache::new()))
        }
    }
    pub fn load_prefab(uid: &i16) -> Arc<PrefabGameObject> {
        unsafe {
            let Some(asset_cache) = &ASSET_CACHE else {
                panic!();
            };
            let Some(asset_database) = &ASSET_DATABASE else {
                panic!();
            };
            let Ok(asset_database) = asset_database.lock() else {
                panic!();
            };
            let Ok(mut asset_cache) = asset_cache.lock() else {
                panic!();
            };

            if let Some(cached_asset) = asset_cache.try_get_asset_prefab(&uid) {
                return cached_asset;
            }

            // not in cache so fetch the asset
            let data = asset_database.fetch_asset(uid);
            if data.len() == 0 {
                panic!("No data for {}!", uid);
            }

            let Ok(asset) = serde_yaml::from_slice::<PrefabGameObject>(&data) else {
                panic!("Failed to unwrap for {}!", uid);
            };

            let arc_asset = Arc::new(asset);

            // add the new data to the cache
            asset_cache.try_store_asset_prefab_asset(&uid, arc_asset.clone());

            // return the asset
            return arc_asset;
        }
    }
    // load - from path
    pub fn load_texture_from_path(path: &i16) -> Arc<TextureAsset> {
        unsafe {
            let Some(asset_cache) = &ASSET_CACHE else {
                panic!();
            };
            let Some(asset_database) = &ASSET_DATABASE else {
                panic!();
            };

            let Ok(asset_database) = asset_database.lock() else {
                panic!();
            };
            let Ok(mut asset_cache) = asset_cache.lock() else {
                panic!();
            };

            // try to get the asset from the cache
            if let Some(cached_asset) = asset_cache.try_get_asset_texture(path) {
                return cached_asset;
            }

            // let data = File::read(path);
            // if data.len() == 0 {
            //     eprintln!("Something went wrong and data came back as empty for path {}", path);
            //     panic!();
            // }

            let data = asset_database.fetch_asset(path);
            let result = Self::unwrap_texture(&data);
            let Ok(result) = result else {
                eprintln!("Something went wrong: {}", result.err().unwrap());
                panic!();
            };

            let asset = Arc::new(result);
            // add the new data to the cache
            asset_cache.try_store_asset_texture(&path, asset.clone());
            // return asset
            asset
        }
    }

    // load - from database
    pub fn load_model_static_from_database(uid: &i16) -> Arc<ModelAsset> {
        let mut all_shaders = HashMap::new();
        all_shaders.insert("assets/shader/my_shader.shader".to_string(), AssetLoader::load_shader_desc(&ASSET_UID_SHADER_LIT));
        all_shaders.insert("assets/shader/unlit_shader.shader".to_string(), AssetLoader::load_shader_desc(&ASSET_UID_SHADER_UNLIT));

        unsafe {
            let Some(asset_cache) = &ASSET_CACHE else {
                panic!();
            };
            let Some(asset_database) = &ASSET_DATABASE else {
                panic!();
            };

            let Ok(asset_database) = asset_database.lock() else {
                panic!();
            };
            let Ok(mut asset_cache) = asset_cache.lock() else {
                panic!();
            };
            // try to get the asset from the cache
            if let Some(cached_asset) = asset_cache.try_get_asset_model(&uid) {
                return cached_asset;
            }

            // not in cache so fetch the asset
            let data = asset_database.fetch_asset(uid);
            if data.len() == 0 {
                panic!("No data for {}!", uid);
            }

            // unwrap
            let unwrapped_gltf = Self::unwrap_gltf(data.as_slice(), all_shaders).unwrap();
            let asset = Arc::new(ModelAsset::new(unwrapped_gltf.0, unwrapped_gltf.1));

            // add the new data to the cache
            asset_cache.try_store_asset_model(&uid, asset.clone());

            // return the asset
            return asset;
        }
    }
    pub fn load_model_animated_from_database(uid: &i16) -> Arc<ModelAssetAnimated> {
        let shader_desc: Arc<ShaderDesc>;
        {
            //create a material
            shader_desc = AssetLoader::load_shader_desc(&ASSET_UID_SHADER_LIT);
        }
        unsafe {
            let Some(asset_database) = &ASSET_DATABASE else {
                panic!();
            };

            let Ok(asset_database) = asset_database.lock() else {
                panic!();
            };

            // let data = asset_database.fetch_asset(uid.clone());
            let data = asset_database.fetch_asset(uid);
            let spine_data = Self::unwrap_spine(data.as_slice());
            let Ok(spine_data) = spine_data else {
                panic!("Err {}", spine_data.err().unwrap());
            };

            let mut material = Material::new("Mat", shader_desc.clone(), false);
            material.set_texture_with_label(Some(Arc::new(spine_data.2)), "diffuse");
            material.finalize();
            // create the asset
            let x = Arc::new(ModelAssetAnimated::new(Arc::new(material), spine_data.1.clone(), Arc::new(AnimationStateData::new(spine_data.1.clone()))));

            return x;
        }
    }
    pub fn load_font_asset(uid: &i16) -> Arc<FontAsset> {
        let font_asset: Arc<FontAsset>;

        {
            let my_struct: FontDesc;
            unsafe {
                let Some(asset_cache) = &ASSET_CACHE else {
                    panic!();
                };
                let Ok(mut asset_cache) = asset_cache.lock() else {
                    panic!();
                };
                let Some(asset_database) = &ASSET_DATABASE else {
                    panic!();
                };

                let Ok(asset_database) = asset_database.lock() else {
                    panic!();
                };

                if let Some(cached) = asset_cache.try_get_asset_font_asset(uid) {
                    return cached.clone();
                }

                // let file = File::read(uid);
                let file = asset_database.fetch_asset(uid);
                let file = String::from_utf8(file).unwrap();
                // let json: serde_json::Value = serde_json::from_str(&file).expect("file should be proper JSON");
                my_struct = serde_json::from_str::<FontDesc>(&file).unwrap();
            }
            {
                font_asset = Arc::new(FontAsset::new(Arc::new(my_struct)));
            }
        }
        unsafe {
            let Some(asset_cache) = &ASSET_CACHE else {
                panic!();
            };
            let Ok(mut asset_cache) = asset_cache.lock() else {
                panic!();
            };

            asset_cache.try_store_asset_font_asset(uid, font_asset.clone());
        }

        font_asset
    }
    // unwrap
    pub fn unwrap_texture(data: &[u8]) -> Result<TextureAsset, Box<dyn Error>> {
        let image: image::DynamicImage = image::load_from_memory(&data).unwrap();
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
    pub fn unwrap_gltf(data: &[u8], shaders: HashMap<String, Arc<ShaderDesc>>) -> Option<(Vec<Arc<Mesh>>, Vec<Arc<Material>>)> {
        // Import GLTF directly from memory slice
        let (gltf, buffers, images) = gltf::import_slice(data).ok()?;

        let mut all_meshes = Vec::with_capacity(gltf.meshes().len());
        let mut all_materials = Vec::with_capacity(gltf.materials().len());

        // --- Materials ---
        if gltf.materials().count() == 0 {
            let s = shaders.get("assets/shader/unlit_shader.shader").unwrap();

            // let shader_desc = AssetLoader::load_shader_desc("assets/shader/unlit_shader.shader");
            all_materials.push(Arc::new(Material::new("gltf", s.clone(), true)));
        } else {
            for material in gltf.materials() {
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

                let shader_desc: Arc<ShaderDesc>;
                if material.name().unwrap().starts_with("lit:") {
                    shader_desc = shaders
                        .get("assets/shader/my_shader.shader")
                        .unwrap()
                        .clone();

                    // shader_desc = AssetLoader::load_shader_desc("assets/shader/my_shader.shader");
                } else if material.name().unwrap().starts_with("unlit:") {
                    shader_desc = shaders
                        .get("assets/shader/unlit_shader.shader")
                        .unwrap()
                        .clone();

                    // shader_desc = AssetLoader::load_shader_desc("assets/shader/unlit_shader.shader");
                } else {
                    shader_desc = shaders
                        .get("assets/shader/my_shader.shader")
                        .unwrap()
                        .clone();

                    // shader_desc = AssetLoader::load_shader_desc("assets/shader/my_shader.shader");
                }

                let mut mat = Material::new("gltf", shader_desc.clone(), false);
                mat.set_texture_with_label(Some(Arc::new(texture_asset)), "diffuse");
                mat.finalize();
                all_materials.push(Arc::new(mat));
            }
        }

        // --- Meshes ---
        for mesh in gltf.meshes() {
            let mesh_name = mesh.name().unwrap_or("Unnamed");

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
    pub fn load_shader_module(device: &Device, path: &i16) -> Arc<ShaderModule> {
        unsafe {
            let Some(asset_cache) = &ASSET_CACHE else {
                panic!();
            };
            let Ok(mut asset_cache) = asset_cache.lock() else {
                panic!();
            };
            let Some(asset_database) = &ASSET_DATABASE else {
                panic!();
            };

            let Ok(asset_database) = asset_database.lock() else {
                panic!();
            };

            // try to get the asset from the cache
            if let Some(cached_asset) = asset_cache.try_get_asset_shader_module(&path) {
                return cached_asset;
            }

            let data = asset_database.fetch_asset(path);
            let string = String::from_utf8(data).unwrap();

            // assetd
            // let contents = fs::read_to_string(path).expect("Should have been able to read the file");
            let module = device.create_shader_module(egui_wgpu::wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: egui_wgpu::wgpu::ShaderSource::Wgsl(string.into()),
            });
            //  let module = device.create_shader_module(egui_wgpu::wgpu::ShaderModuleDescriptor {
            //     label: Some(path),
            //     source: egui_wgpu::wgpu::ShaderSource::Wgsl(contents.into()),
            // });

            let asset = Arc::new(module);

            asset_cache.try_store_asset_shader_module(path, asset.clone());

            return asset;
        }
    }
    pub fn load_shader_desc(path: &i16) -> Arc<ShaderDesc> {
        unsafe {
            let Some(asset_cache) = &ASSET_CACHE else {
                panic!();
            };
            let Some(asset_database) = &ASSET_DATABASE else {
                panic!();
            };

            let Ok(asset_database) = asset_database.lock() else {
                panic!();
            };
            let Ok(mut asset_cache) = asset_cache.lock() else {
                panic!();
            };

            // try to get the asset from the cache
            if let Some(cached_asset) = asset_cache.try_get_asset_shader_desc(path) {
                return cached_asset;
            }

            let data = &asset_database.fetch_asset(path);

            // load
            // let file = fs::File::open(path).expect("file should open read only");
            // let json: serde_json::Value = serde_json::from_reader(file).expect("file should be proper JSON");

            let json: serde_json::Value = serde_json::from_slice(data).expect("file should be proper JSON");
            let my_struct: ShaderDesc = serde_json::from_str(&json.to_string()).unwrap();

            let asset = Arc::new(my_struct);
            asset_cache.try_store_asset_shader_desc(path, asset.clone());
            return asset;
        }
    }
    pub fn load_font_asset_from_path(path: &str) -> Arc<FontDesc> {
        let file = File::read(path);
        let json: serde_json::Value = serde_json::from_slice(file.as_slice()).expect("file should be proper JSON");
        let my_struct: FontDesc = serde_json::from_str(&json.to_string()).unwrap();
        let asset = Arc::new(my_struct);
        asset
    }
}

pub struct FontAsset {
    desc: Arc<FontDesc>,
    material: Arc<Material>,
    mesh: HashMap<char, Arc<ModelAsset>>,
    glyph_width: f32,
    glyph_height: f32,
}

impl FontAsset {
    pub fn new(desc: Arc<FontDesc>) -> FontAsset {
        // FontAsset {
        //     desc
        // }

        // let texture = AssetLoader::load_texture_from_path(&File::join_path(&File::get_built_in_asset_path(), &desc.texture_path));
        // let shader = AssetLoader::load_shader_desc(&File::join_path(&File::get_built_in_asset_path(), &desc.shader_path));

        let texture = AssetLoader::load_texture_from_path(&ASSET_UID_TEXTURE_FONT_ATLAS);
        let shader = AssetLoader::load_shader_desc(&ASSET_UID_SHADER_UNLIT);

        let w = texture.texture.width() as f32;
        let h = texture.texture.height() as f32;

        let padding_left_01 = desc.padding_left as f32 / w;
        let padding_right_01 = desc.padding_right as f32 / w;
        let padding_top_01 = desc.padding_top as f32 / h;
        let padding_bottom_01 = desc.padding_bottom as f32 / h;

        // Glyph UV layout
        let glyph_width = (1.0 - padding_left_01 - padding_right_01) / desc.columns as f32;
        let glyph_height = (1.0 - padding_top_01 - padding_bottom_01) / desc.rows as f32;

        let mut mesh = HashMap::new();

        let mut material = Material::new("font asset", shader, false);
        material.set_texture_with_label(Some(texture), "diffuse");
        material.finalize();
        let material = Arc::new(material);

        for (index, ch) in desc.char_order.chars().enumerate() {
            let col = index as i32 % desc.columns;
            let row = index as i32 / desc.columns;

            let u_min = padding_left_01 + col as f32 * glyph_width;
            let v_min = padding_top_01 + row as f32 * glyph_height;
            let u_max = u_min + glyph_width;
            let v_max = v_min + glyph_height;
            // let m =
            let vertices = vec![
                Vertex {
                    position: [0.0, 0.0, 0.0],
                    normal: [0.0, 0.0, 1.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                    uv0: [u_min, v_max],
                    uv1: [0.0, 0.0],
                },
                Vertex {
                    position: [1.0, 0.0, 0.0],
                    normal: [0.0, 0.0, 1.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                    uv0: [u_max, v_max],
                    uv1: [0.0, 0.0],
                },
                Vertex {
                    position: [1.0, 1.0, 0.0],
                    normal: [0.0, 0.0, 1.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                    uv0: [u_max, v_min],
                    uv1: [0.0, 0.0],
                },
                Vertex {
                    position: [0.0, 1.0, 0.0],
                    normal: [0.0, 0.0, 1.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                    uv0: [u_min, v_min],
                    uv1: [0.0, 0.0],
                },
            ];
            let m = Arc::new(Mesh::new(format!("glyph_{}", ch), vertices, vec![0, 1, 2, 0, 2, 3], Matrix4x4::default()));

            mesh.insert(ch, Arc::new(ModelAsset::new(vec![m], vec![material.clone()])));
        }

        FontAsset {
            desc: desc,
            material: material,
            mesh,
            glyph_width,
            glyph_height,
        }
    }

    pub fn glyph_width(&self) -> f32 {
        self.glyph_width
    }
    pub fn glyph_height(&self) -> f32 {
        self.glyph_height
    }
    pub fn material(&self) -> Arc<Material> {
        self.material.clone()
    }
    pub fn mesh_all() {}
    pub fn mesh_for_char(&self, char: char) -> Arc<ModelAsset> {
        let Some(cached) = self.mesh.get(&char) else {
            return Arc::new(ModelAsset::new(vec![], vec![]));
        };

        return cached.clone();
    }
}
#[derive(Serialize, Deserialize)]
pub struct PrefabGameObject {
    pub name: String,
    pub components: Vec<PrefabComponent>,
    pub children: Vec<PrefabGameObject>,
}
#[derive(Serialize, Deserialize)]
pub struct PrefabComponent {
    pub r#type: String,
    pub fields: Vec<String>,
}
