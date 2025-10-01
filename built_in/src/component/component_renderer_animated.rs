use core::{
    collections::{
        matrix4x4::Matrix4x4,
        mesh::{Mesh, Vertex},
    },
    io::model_asset_animated::ModelAssetAnimated,
};
use rusty_spine::{AnimationState, Skeleton};
use std::sync::Arc;

// #[derive(Clone)]
pub struct RendererAnimated {
    skeleton: Option<Skeleton>,
    state: Option<AnimationState>,
    fps: i32,
    last_update: f64,
    pub asset: Option<Arc<ModelAssetAnimated>>,
    pub mesh: Vec<Arc<Mesh>>,
}

impl RendererAnimated {
    pub fn default() -> RendererAnimated {
        RendererAnimated {
            asset: None,
            skeleton: None,
            state: None,
            mesh: vec![],
            fps: 24,
            last_update: -9999.0,
        }
    }
    /// Set the current playing animation
    pub fn set_animation(mut self, name: &str, looping: bool) -> Self {
        if let Some(asset_animated) = &self.asset {
            if let Some(state) = &mut self.state {
                if let Some(animation) = asset_animated.skeleton_data.find_animation(name) {
                    // set animation
                    state.set_animation(0, &animation, looping);

                    // update mesh
                    self.update_mesh(self.last_update);
                }
            }
        }
        self
    }
    /// Set the visible skin
    pub fn set_skin(mut self, name: &str) -> Self {
        if let Some(skeleton) = &mut self.skeleton {
            let resukt = skeleton.set_skin_by_name(name);
            match resukt {
                Err(e) => println!("{}", e),
                _ => {}
            }

            // update mesh
            self.update_mesh(self.last_update);
        }
        self
    }
    /// Set the asset
    pub fn set_asset(mut self, asset: Option<Arc<ModelAssetAnimated>>) -> Self {
        // set the asset
        self.asset = asset;

        // if asset has
        if let Some(asset_animated) = &self.asset {
            // create state
            let state = AnimationState::new(asset_animated.state_data.clone());

            // create skeleton
            let mut skeleton = Skeleton::new(asset_animated.skeleton_data.clone());
            skeleton.set_to_setup_pose();

            // set for asset
            self.state = Some(state);
            self.skeleton = Some(skeleton);

            // update mesh
            self.update_mesh(self.last_update);
        } else {
            // clear
            self.state = None;
            self.skeleton = None;
        }

        // return this
        self
    }
    /// Updates the mesh to match the current animation state. This should only be called ONCE per frame.
    pub fn update_mesh(&mut self, time: f64) {
        let delta = time - self.last_update;
        if delta < 1.0 / (self.fps as f64) {
            return;
        }

        self.last_update = time;

        let Some(skeleton) = &mut self.skeleton else {
            panic!();
        };
        let Some(state) = &mut self.state else {
            panic!();
        };

        // update skeleton
        skeleton.update(delta as f32);
        // update state
        state.update(delta as f32);
        // apply all changes
        state.apply(skeleton);
        // update all transforms
        skeleton.update_world_transform(rusty_spine::Physics::None);

        let mut z: f32 = 0.0;
        let mut out = Vec::new();
        // iterate slots in draw order so attachments are in correct order
        for slot in skeleton.draw_order() {
            z += -0.01;
            if let Some(attachment) = slot.attachment() {
                // Region attachments (quads)
                if let Some(region) = attachment.as_region() {
                    // region.compute_world_vertices writes 4 vertices -> 8 floats (x,y)
                    let mut world = [0f32; 8]; // 4 * (x,y)
                    // offset = 0, stride = 2 (tight x,y)
                    unsafe {
                        region.compute_world_vertices(&slot, &mut world, 0, 2);
                    }

                    let uvs = region.uvs(); // [f32; 8]

                    let mut verts = Vec::with_capacity(4);
                    for i in 0..4 {
                        let x = world[i * 2];
                        let y = world[i * 2 + 1];
                        let uvx = uvs[i * 2];
                        let uvy = uvs[i * 2 + 1];

                        verts.push(Vertex {
                            uv0: [uvx, uvy],
                            uv1: [uvx, uvy],
                            position: [x, y, z],
                            normal: [0.0, 0.0, 1.0],
                            color: [1.0, 1.0, 1.0, 1.0], // tint handling left to you
                        });
                    }

                    // standard quad indices (two tris). Adjust winding if needed.
                    let indices = vec![0u32, 1, 2, 2, 3, 0];

                    // Adjust this call to match your Mesh constructor
                    let mesh = Mesh::new(String::from("region"), verts, indices, Matrix4x4::default());
                    out.push(Arc::new(mesh));
                    continue;
                }

                // Mesh attachments (deformable meshes)
                if let Some(mesh_att) = attachment.as_mesh() {
                    let world_len = mesh_att.world_vertices_length() as usize; // number of floats (x,y, ...)
                    // We're going to request the full set of world vertex values into a tight array
                    let mut world = vec![0f32; world_len];

                    // MeshAttachment::compute_world_vertices(start, count, world, offset, stride)
                    // start = 0, count = world_len (number of floats), offset = 0, stride = 2 for tight x,y.
                    // NOTE: compute_world_vertices is unsafe in rusty_spine; the slot must be the same slot the attachment originated from.
                    unsafe {
                        mesh_att.compute_world_vertices(&slot, 0, world_len as i32, &mut world, 0, 2);
                    }

                    // UVs are stored as floats (u,v per vertex). Their length equals world_len (2 floats per vertex).
                    let uvs: &[f32] = unsafe {
                        // mesh_att.uvs() returns *mut c_float; interpret as f32 pointer.
                        std::slice::from_raw_parts(mesh_att.uvs() as *const f32, world_len)
                    };

                    // triangles: pointer + count
                    let tri_count = mesh_att.triangles_count() as usize;
                    let tris: &[u16] = unsafe { std::slice::from_raw_parts(mesh_att.triangles(), tri_count) };

                    let vcount = world_len / 2;
                    let mut verts = Vec::with_capacity(vcount);
                    for i in 0..vcount {
                        let x = world[i * 2];
                        let y = world[i * 2 + 1];
                        let uvx = uvs[i * 2];
                        let uvy = uvs[i * 2 + 1];

                        verts.push(Vertex {
                            uv0: [uvx, uvy],
                            uv1: [uvx, uvy],
                            position: [x, y, z],
                            normal: [0.0, 0.0, 1.0],
                            color: [1.0, 1.0, 1.0, 1.0],
                        });
                    }

                    // convert u16 indices into whatever your mesh index type is (u32 assumed here)
                    let indices: Vec<u32> = tris.iter().map(|&t| t as u32).collect();

                    // let matrix = Matrix4x4::new(mesh_att., rot, scale)

                    let mesh = Mesh::new(String::from("mesh"), verts, indices, Matrix4x4::default());
                    out.push(Arc::new(mesh));
                    continue;
                }

                // other attachment types ignored here
            }
        }

        self.mesh = out;
    }
}
