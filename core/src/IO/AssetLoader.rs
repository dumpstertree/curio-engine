use std::collections::HashMap;
use std::fs;
use std::sync::Arc;

use egui_wgpu::wgpu::Device;
use egui_wgpu::wgpu::ShaderModule;

// use crate::system_adapters::adapter_system_gpu::SystemGPU;
use crate::Collections::material::Material;
use crate::Collections::material::ShaderDesc;
use crate::Collections::Mesh::Mesh;
use crate::Collections::Mesh::Vertex;

use super::model_asset::ModelAsset;
use super::texture_asset::TextureAsset;

pub struct AssetLoader {
    asset_cache: HashMap<String, Arc<ModelAsset>>, // shader_cache: ShaderCache<'a>,
                                                   // path_texture: String,
                                                   // path_model: String,
                                                   // device: &'a Device,
                                                   // queue: &'a egui_wgpu::wgpu::Queue,
                                                   // state: &'a State,
}
// impl ISystemComponent for AssetLoader {
//     fn init(&mut self, asset_loader: &mut AssetLoader, gs: &mut crate::Window::SystemWindow::GameState) {}
// }
// construction
impl AssetLoader {
    pub fn new() -> AssetLoader {
        AssetLoader { asset_cache: HashMap::new() }
    }
    // pub fn new<'a>(shader_cache: ShaderCache<'a>, device: &'a Device, queue: &'a egui_wgpu::wgpu::Queue) -> AssetLoader<'a> {
    //     AssetLoader {
    //         shader_cache: shader_cache,
    //         path_texture: String::from("Assets/Texture"),
    //         path_model: String::from("assets"),
    //         device: device,
    //         queue: queue,
    //     }
    // }
    // pub fn new<'a>(shader_cache: ShaderCache<'a>, state: &'a State) -> AssetLoader {
    //     AssetLoader {
    //         // shader_cache: shader_cache,
    //         // path_texture: String::from("Assets/Texture"),
    //         // path_model: String::from("assets"),
    //         // state: state,
    //     }
    // }
}

const PATH_MESH: &str = "assets/mesh";
// private
impl AssetLoader {
    pub fn clear_cache(&mut self) {
        self.asset_cache.clear();
    }
    pub fn reduce_cache() {}
    pub fn load_png(path: String) -> Option<TextureAsset> {
        // unwrap into vec of bytes
        let bytes = std::fs::read(path);

        // unwrap value
        let bytes = bytes.unwrap();

        // convert bytes to jpg
        let image: Result<image::DynamicImage, image::ImageError> = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png);

        // unwrap or return null
        match image {
            Ok(x) => Some(x),
            _ => panic!("Failed to load image"),
        };

        // fallthrough
        None
    }

    // const PATH_TEXTURE: &str = "Assets/Texture";
    // const PATH_MESH: &str = "assets";

    pub fn load_jpg(_: &str) -> Option<TextureAsset> {
        // // get the path using env path as base
        // let full_path = std::path::Path::new(PATH_MESH).join(path);

        // // unwrap into vec of bytes
        // let bytes = std::fs::read(full_path);

        // // unwrap value
        // let bytes = bytes.unwrap();

        // // convert bytes to jpg
        // let image: Result<image::DynamicImage, image::ImageError> = image::load_from_memory_with_format(&bytes, image::ImageFormat::Jpeg);

        // unwrap or return null
        // match image {
        //     Ok(x) => {
        //         Texture_asset::new(x.width() as i32, x.height() as i32, bytes);
        //     }
        //     _ => panic!("Failed to load image"),
        // };

        // fallthrough
        None
    }

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
    pub fn load_gltf<'a>(&mut self, path: &str) -> Option<Arc<ModelAsset>> {
        // return cached
        if self.asset_cache.contains_key(path) {
            return Some(self.asset_cache[path].clone());
        }
        // build new

        // let device = SystemGPU::get_device();
        // let queue = SystemGPU::get_queue();
        // let Some(device) = &sys.device else { return None };
        // let Some(queue) = &sys.queue else { return None };
        // get the path using env path as base
        let full_path = std::path::Path::new(PATH_MESH).join(path);
        let z = gltf::import(&full_path);

        match z {
            Ok(x) => {
                let gltf = x.0;
                let buffers = x.1;
                let images = x.2;

                // declare output mesh
                let mut all_mesh: Vec<Mesh> = Vec::new();
                let mut all_material: Vec<Material> = Vec::new();
                // let shader_desc = ShaderDesc::new_from_module("../shader.wgsl", vec![ShaderTextureDesc::new("diffuse")]);

                let shader_desc = AssetLoader::load_shader_desc("assets/shader/my_shader.shader");

                if gltf.materials().len() == 0 {
                    all_material.push(Material::new(shader_desc.clone()));
                } else {
                    for material in gltf.materials() {
                        let pbr = material.pbr_metallic_roughness();

                        let texture_asset: TextureAsset; //= //Texture_asset::new(material.name(), device, queue, width as i32, height as i32, bytes);
                        if let Some(t) = pbr.base_color_texture() {
                            let image2 = &images[t.texture().index()];
                            let mut p = image2.pixels.clone();

                            if image2.format == gltf::image::Format::R8G8B8 {
                                for i in 0..(image2.width * image2.height) {
                                    p.insert((3 + (i * 4)) as usize, 0);
                                }
                            }

                            texture_asset = TextureAsset::new_from_buffer(None, image2.width, image2.height, &p[..]);
                        } else {
                            texture_asset = TextureAsset::default();
                        }

                        let mut m = Material::new(shader_desc.clone());
                        m.set_texture_with_label(Some(texture_asset), "diffuse");
                        all_material.push(m);
                    }
                }
                // }
                // iterate over gltf
                for mesh in gltf.meshes() {
                    println!("add mesh: {}", mesh.name().unwrap());
                    for primitive in mesh.primitives() {
                        // allows you to read data from the primitive
                        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                        // output values - will be set to empty if reference has no value
                        let mut indices: Vec<u32> = Vec::new();
                        let mut verticies: Vec<Vertex> = Vec::new();

                        // assume the positions are the count of verticies
                        match reader.read_positions() {
                            Some(positions) => {
                                for _ in 0..positions.len() {
                                    verticies.push(Vertex::default());
                                }
                            }
                            _ => return None,
                        }
                        // update the normal - if exists
                        match reader.read_positions() {
                            Some(positions) => {
                                let mut i = 0;
                                for j in positions {
                                    verticies[i].position[0] = j[0]; // x
                                    verticies[i].position[1] = j[1]; // y
                                    verticies[i].position[2] = j[2]; // z
                                    i += 1;
                                }
                            }
                            _ => {}
                        }
                        // // update the normal - if exists
                        match reader.read_normals() {
                            Some(normals) => {
                                let mut i = 0;
                                for j in normals {
                                    verticies[i].normal[0] = j[0]; // x
                                    verticies[i].normal[1] = j[1]; // y
                                    verticies[i].normal[2] = j[2]; // z
                                    i += 1;
                                }
                            }
                            _ => {}
                        }
                        // update uv 0 - if exists
                        match reader.read_tex_coords(0) {
                            Some(uv) => {
                                let mut i = 0;
                                for j in uv.into_f32() {
                                    verticies[i].uv0[0] = j[0]; // u
                                    verticies[i].uv0[1] = j[1]; // v
                                    i += 1;
                                }
                            }
                            _ => {}
                        }
                        // update uv 1 - if exists
                        match reader.read_tex_coords(1) {
                            Some(uv) => {
                                let mut i = 0;
                                for j in uv.into_f32() {
                                    verticies[i].uv1[0] = j[0]; // u
                                    verticies[i].uv1[1] = j[1]; // v
                                    i += 1;
                                }
                            }
                            _ => {}
                        }
                        // update color - if exists
                        match reader.read_colors(0) {
                            Some(color) => {
                                let mut i = 0;
                                for j in color.into_rgba_f32() {
                                    verticies[i].color[0] = j[0]; // r
                                    verticies[i].color[1] = j[1]; // g
                                    verticies[i].color[2] = j[2]; // b
                                    verticies[i].color[3] = j[3]; // a
                                    i += 1;
                                }
                            }
                            _ => {}
                        }

                        // set indicies
                        match reader.read_indices() {
                            Some(indicies) => {
                                for i in indicies.into_u32() {
                                    indices.push(i);
                                }
                            }
                            _ => {
                                println!("Indicies failed");
                                return None;
                            }
                        }

                        let m = mesh.name().unwrap();
                        let mesh_id = full_path.join(m);
                        let mesh_id = mesh_id.to_str().unwrap();

                        // add mesh to list
                        all_mesh.push(Mesh::new(String::from(mesh_id), verticies, indices));
                    }
                }
                let asset = Arc::new(ModelAsset::new(all_mesh, all_material));

                self.asset_cache.insert(String::from(path), asset.clone());

                return Some(asset);
            }
            Err(e) => {
                println!("{}", e);
                return None;
            }
        }
    }
}

// pub struct ShaderCache<'a> {
//     device: &'a State,
//     cache: HashMap<String, ShaderModule>,
// }
// impl ShaderCache<'_> {
//     pub fn new<'a>(device: &State) -> ShaderCache {
//         ShaderCache {
//             device: device,
//             cache: HashMap::new(),
//         }
//     }
//     pub fn load(&self, path: &str) -> ShaderModule {
//         // get key for path
//         let key = String::from(path);

//         // if not cached create
//         // let is_cached = &self.cache.contains_key(&key);
//         // if !is_cached {
//         // load shader using saved device
//         let shader = self.device.box_device.create_shader_module(wgpu::ShaderModuleDescriptor {
//             label: Some("Shader"),
//             source: egui_wgpu::wgpu::ShaderSource::Wgsl(include_str!("../shader.wgsl").into()),
//         });

//         shader

//         // &self.cache.insert(key.clone(), shader);
//         // }

//         //
//         // &self.cache[&key]
//     }
// }
