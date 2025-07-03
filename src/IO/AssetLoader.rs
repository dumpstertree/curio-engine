use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::fs;
use std::io::BufRead;

use gltf::mesh::util::colors::RgbaU8;
use image::DynamicImage;
use image::ImageBuffer;
use image::ImageFormat;
use image::RgbImage;
use image::Rgba;
use wgpu::Device;
use wgpu::Queue;
use wgpu::ShaderModule;

use crate::texture;
use crate::Collections::material::Material;
use crate::Collections::material::ShaderDesc;
use crate::Collections::material::ShaderTextureDesc;
use crate::Collections::Mesh::Mesh;
use crate::Collections::Mesh::Vertex;
use crate::Window::state::State;
use crate::Window::SystemWindow::SYS_GPU;

use super::model_asset::Model_asset;
use super::texture_asset::Texture_asset;

pub struct AssetLoader {
    // shader_cache: ShaderCache<'a>,
    // path_texture: String,
    // path_model: String,
    // device: &'a Device,
    // queue: &'a wgpu::Queue,
    // state: &'a State,
}
// impl ISystemComponent for AssetLoader {
//     fn init(&mut self, asset_loader: &mut AssetLoader, gs: &mut crate::Window::SystemWindow::GameState) {}
// }
// construction
impl AssetLoader {
    // pub fn new<'a>(shader_cache: ShaderCache<'a>, device: &'a Device, queue: &'a wgpu::Queue) -> AssetLoader<'a> {
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

const PATH: &str = "assets";

// private
impl AssetLoader {
    pub fn clear_cache() {}
    pub fn reduce_cache() {}
    pub fn load_png(path: String) -> Option<Texture_asset> {
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

    pub fn load_jpg(path: &str) -> Option<Texture_asset> {
        // get the path using env path as base
        let full_path = std::path::Path::new(PATH).join(path);

        // unwrap into vec of bytes
        let bytes = std::fs::read(full_path);

        // unwrap value
        let bytes = bytes.unwrap();

        // convert bytes to jpg
        let image: Result<image::DynamicImage, image::ImageError> = image::load_from_memory_with_format(&bytes, image::ImageFormat::Jpeg);

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
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(path),
            source: wgpu::ShaderSource::Wgsl(contents.into()),
        })
    }
    pub fn load_shader_desc(path: &str) -> ShaderDesc {
        let file = fs::File::open(path).expect("file should open read only");
        let json: serde_json::Value = serde_json::from_reader(file).expect("file should be proper JSON");
        let my_struct: ShaderDesc = serde_json::from_str(&json.to_string()).unwrap();
        my_struct
    }
    pub fn load_gltf<'a>(path: &str) -> Option<Model_asset> {
        let sys = SYS_GPU.lock().unwrap();
        let Some(device) = &sys.device else { return None };
        let Some(queue) = &sys.queue else { return None };
        // get the path using env path as base
        let full_path = std::path::Path::new(PATH).join(path);
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

                let shader_desc = AssetLoader::load_shader_desc("assets/my_shader.shader");

                if gltf.materials().len() == 0 {
                    all_material.push(Material::new(shader_desc.clone(), device));
                } else {
                    for material in gltf.materials() {
                        let pbr = material.pbr_metallic_roughness();

                        let texture_asset: Texture_asset; //= //Texture_asset::new(material.name(), device, queue, width as i32, height as i32, bytes);
                        if let Some(t) = pbr.base_color_texture() {
                            let image2 = &images[t.texture().index()];
                            let mut p = image2.pixels.clone();

                            if image2.format == gltf::image::Format::R8G8B8 {
                                for i in 0..(image2.width * image2.height) {
                                    p.insert((3 + (i * 4)) as usize, 0);
                                }
                            }

                            texture_asset = Texture_asset::new_from_buffer(None, device, queue, image2.width, image2.height, &p[..]);
                        } else {
                            texture_asset = Texture_asset::default(device, queue);
                        }

                        let mut m = Material::new(shader_desc.clone(), device);
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
                                for i in 0..positions.len() {
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

                // output asset
                return Some(Model_asset::new(all_mesh, all_material));
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
//             source: wgpu::ShaderSource::Wgsl(include_str!("../shader.wgsl").into()),
//         });

//         shader

//         // &self.cache.insert(key.clone(), shader);
//         // }

//         //
//         // &self.cache[&key]
//     }
// }
