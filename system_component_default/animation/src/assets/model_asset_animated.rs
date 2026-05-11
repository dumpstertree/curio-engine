use curio_core::{
    AssetCommon, AssetCommonFromBits, Material, Matrix4x4, Mesh, ModelAsset, ShaderDesc, TextureAsset, Vertex,
    f32::FloatExtras,
    io::asset_loader::{ASSET_UID_SHADER_LIT, AssetLoader},
};

use egui::ahash::HashMap;
use rayon::prelude::*;
use rusty_spine::{AnimationState, AnimationStateData, Atlas, Skeleton, SkeletonData, SkeletonJson};
use std::{error::Error, io::Cursor};
use std::{sync::Arc, time::Instant};
use zip::ZipArchive;

pub struct ModelAssetAnimated {
    material: Arc<Material>,
    skeleton_data: Arc<SkeletonData>,
    state_data: Arc<AnimationStateData>,
    cached_animation_frames: HashMap<String, Arc<AnimationAsset>>,
}

impl ModelAssetAnimated {
    pub fn material(&self) -> Arc<Material> {
        self.material.clone()
    }
    pub fn set_material(&mut self, material: Arc<Material>) {
        self.material = material;
    }
    pub fn instantiate(&self) -> ModelAssetAnimated {
        ModelAssetAnimated {
            material: self.material.clone(),
            skeleton_data: self.skeleton_data.clone(),
            state_data: self.state_data.clone(),
            cached_animation_frames: self.cached_animation_frames.clone(),
        }
    }

    pub fn new(material: Arc<Material>, skeleton_data: Arc<SkeletonData>, state_data: Arc<AnimationStateData>) -> ModelAssetAnimated {
        let mut m = ModelAssetAnimated {
            material,
            skeleton_data,
            state_data,
            cached_animation_frames: HashMap::default(),
        };
        let s = Instant::now();
        m.finalize();
        println!("elapsed {}", s.elapsed().as_secs_f32());
        m
    }

    pub fn get_animation(&self, name: &str) -> Arc<AnimationAsset> {
        self.cached_animation_frames
            .get(name)
            .cloned()
            .unwrap_or_else(|| Arc::new(AnimationAsset::new(Vec::new())))
    }

    pub fn finalize(&mut self) {
        // pull out consts
        const FPS: f32 = 24.0;
        const DELTA: f32 = 1.0 / FPS;
        const QUAD_INDICES: [u32; 6] = [0, 1, 2, 2, 3, 0];

        // setup skelton we are using
        let mut skeleton = Skeleton::new(self.skeleton_data.clone());
        skeleton.set_to_setup_pose();

        // Collect animation names up-front (single-threaded)
        let animation_names: Vec<String> = self
            .skeleton_data
            .animations()
            .map(|a| a.name().to_string())
            .collect();

        let material = self.material.clone();
        let skeleton_data = self.skeleton_data.clone();
        let state_data = self.state_data.clone();

        // Bake each animation in parallel
        let baked: Vec<(String, Arc<AnimationAsset>)> = animation_names
            .into_par_iter()
            .filter_map(|name| {
                let animation = skeleton_data.find_animation(&name)?;

                // Thread-local Spine state
                let mut skeleton = Skeleton::new(skeleton_data.clone());
                skeleton.set_to_setup_pose();

                let mut state = AnimationState::new(state_data.clone());
                state.set_animation(0, &animation, false);

                let frame_count = f32::ceil(animation.duration() * FPS).max(1.0) as usize;

                // Scratch buffers (thread-local)
                let mut world_buf: Vec<f32> = Vec::new();
                let mut verts: Vec<Vertex> = Vec::new();

                let mut frames: Vec<Arc<FrameAsset>> = Vec::with_capacity(frame_count);

                for _ in 0..frame_count {
                    skeleton.update(DELTA);
                    state.update(DELTA);
                    state.apply(&mut skeleton);
                    skeleton.update_world_transform();

                    let mut z = 0.0f32;
                    let mut frame_mesh: Vec<Arc<ModelAsset>> = Vec::new();

                    for slot in skeleton.draw_order() {
                        z -= 0.01;

                        let Some(attachment) = slot.attachment() else {
                            continue;
                        };

                        /* ---------- Region ---------- */
                        if let Some(region) = attachment.as_region() {
                            let mut world = [0.0f32; 8];
                            unsafe {
                                region.compute_world_vertices(&*slot.bone(), &mut world, 0, 2);
                            }

                            let uvs = region.uvs();
                            verts.clear();
                            verts.reserve_exact(4);

                            for i in 0..4 {
                                let w = i * 2;
                                verts.push(Vertex {
                                    position: [world[w], world[w + 1], z],
                                    uv0: [uvs[w], uvs[w + 1]],
                                    uv1: [uvs[w], uvs[w + 1]],
                                    normal: [0.0, 0.0, 1.0],
                                    color: [1.0, 1.0, 1.0, 1.0],
                                });
                            }

                            let mesh = Arc::new(Mesh::new("region".into(), verts.clone(), QUAD_INDICES.to_vec(), Matrix4x4::default()));

                            frame_mesh.push(Arc::new(ModelAsset::new(vec![mesh], vec![material.clone()])));
                            continue;
                        }

                        /* ---------- Mesh ---------- */
                        if let Some(mesh_att) = attachment.as_mesh() {
                            let world_len = mesh_att.world_vertices_length() as usize;

                            world_buf.clear();
                            world_buf.resize(world_len, 0.0);

                            unsafe {
                                mesh_att.compute_world_vertices(&slot, 0, world_len as i32, &mut world_buf, 0, 2);
                            }

                            let uvs: &[f32] = unsafe { std::slice::from_raw_parts(mesh_att.uvs() as *const f32, world_len) };

                            let tris: &[u16] = unsafe { std::slice::from_raw_parts(mesh_att.triangles(), mesh_att.triangles_count() as usize) };

                            let vcount = world_len / 2;
                            verts.clear();
                            verts.reserve_exact(vcount);

                            for i in 0..vcount {
                                let w = i * 2;
                                verts.push(Vertex {
                                    position: [world_buf[w], world_buf[w + 1], z],
                                    uv0: [uvs[w], uvs[w + 1]],
                                    uv1: [uvs[w], uvs[w + 1]],
                                    normal: [0.0, 0.0, 1.0],
                                    color: [1.0, 1.0, 1.0, 1.0],
                                });
                            }

                            let indices: Vec<u32> = tris.iter().map(|&i| i as u32).collect();

                            let mesh = Arc::new(Mesh::new("mesh".into(), verts.clone(), indices, Matrix4x4::default()));

                            frame_mesh.push(Arc::new(ModelAsset::new(vec![mesh], vec![material.clone()])));
                        }
                    }

                    frames.push(Arc::new(FrameAsset::new(frame_mesh)));
                }

                Some((name, Arc::new(AnimationAsset::new(frames))))
            })
            .collect();

        // Merge results back (single-threaded)
        self.cached_animation_frames.clear();
        for (name, anim) in baked {
            self.cached_animation_frames.insert(name, anim);
        }
    }
}
impl ModelAssetAnimated {
    fn unwrap_spine(data: &[u8]) -> Result<(Arc<Atlas>, Arc<SkeletonData>, TextureAsset), Box<dyn Error>> {
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
}
impl AssetCommon for ModelAssetAnimated {}
impl AssetCommonFromBits<ModelAssetAnimated> for ModelAssetAnimated {
    fn from_bits(bits: &Vec<u8>) -> ModelAssetAnimated {
        //create a material
        let shader_desc = AssetLoader::load_asset::<ShaderDesc>(&ASSET_UID_SHADER_LIT);
        let spine_data = Self::unwrap_spine(bits);
        let Ok(spine_data) = spine_data else {
            panic!("Err {}", spine_data.err().unwrap());
        };

        let mut material = Material::new("Mat", shader_desc.clone(), false);
        material.set_texture_with_label(Some(Arc::new(spine_data.2)), "diffuse");
        material.finalize();
        // create the asset
        ModelAssetAnimated::new(Arc::new(material), spine_data.1.clone(), Arc::new(AnimationStateData::new(spine_data.1.clone())))
    }
}

pub struct AnimationAsset {
    frames: Vec<Arc<FrameAsset>>,
}

impl AnimationAsset {
    pub fn new(frames: Vec<Arc<FrameAsset>>) -> AnimationAsset {
        AnimationAsset { frames }
    }

    pub fn all_frames(&self) -> &[Arc<FrameAsset>] {
        &self.frames
    }

    pub fn get_frame_num_for_normalized_time(&self, time_01: f32, looping: bool) -> usize {
        if self.frames.is_empty() {
            return 0;
        }

        let t = if looping { time_01.repeat(1.0) } else { time_01.clamp(0.0, 1.0) };

        f32::round((self.frames.len() - 1) as f32 * t) as usize
    }

    pub fn frame_for_index(&self, index: usize) -> Arc<FrameAsset> {
        self.frames
            .get(index)
            .cloned()
            .unwrap_or_else(|| Arc::new(FrameAsset { mesh: Vec::new() }))
    }

    pub fn frame_for_time() {
        todo!()
    }
}

pub struct FrameAsset {
    mesh: Vec<Arc<ModelAsset>>,
}

impl FrameAsset {
    pub fn new(mesh: Vec<Arc<ModelAsset>>) -> FrameAsset {
        FrameAsset { mesh }
    }

    pub fn mesh(&self) -> &[Arc<ModelAsset>] {
        &self.mesh
    }
}
