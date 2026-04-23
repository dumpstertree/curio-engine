use std::{collections::HashMap, sync::Arc};

use crate::{
    assets::asset::AssetCommonFromBits,
    graphics::{
        material::{Material, ShaderDesc},
        mesh::Mesh,
    },
    io::asset_loader::{AssetLoader, ASSET_UID_SHADER_LIT, ASSET_UID_SHADER_UNLIT},
    Matrix4x4, Random, TextureAsset, Vertex,
};

use super::asset::AssetCommon;

//data

#[derive(Clone)]
pub struct ModelAsset {
    pub instance_id: i32,
    pub mesh: Vec<Arc<Mesh>>,
    pub materials: Vec<Arc<Material>>,
}

// construction
impl ModelAsset {
    pub fn new(mesh: Vec<Arc<Mesh>>, materials: Vec<Arc<Material>>) -> ModelAsset {
        ModelAsset {
            instance_id: Random::range_int(-9999999, 99999999),
            mesh,
            materials,
        }
    }
}
// public
impl ModelAsset {}
// private
impl ModelAsset {
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
}
// asset
impl AssetCommon for ModelAsset {}
impl AssetCommonFromBits<ModelAsset> for ModelAsset {
    fn from_bits(bits: &Vec<u8>) -> ModelAsset {
        let mut all_shaders = HashMap::new();
        all_shaders.insert("assets/shader/my_shader.shader".to_string(), AssetLoader::load_asset::<ShaderDesc>(&ASSET_UID_SHADER_LIT));
        all_shaders.insert("assets/shader/unlit_shader.shader".to_string(), AssetLoader::load_asset::<ShaderDesc>(&ASSET_UID_SHADER_UNLIT));

        let x = ModelAsset::unwrap_gltf(bits, all_shaders).unwrap();

        return ModelAsset::new(x.0, x.1);
    }
}
