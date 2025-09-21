use std::sync::Arc;

use core::{
    collections::{
        matrix4x4::Matrix4x4,
        mesh::{Mesh, Vertex},
    },
    io::{model_asset::ModelAsset, model_asset_animated::ModelAssetAnimated},
};

use rusty_spine::{AnimationState, Skeleton};

// #[derive(Clone)]
pub struct Renderer {
    pub asset: Option<Arc<ModelAsset>>,
    pub asset_animated: Option<Arc<ModelAssetAnimated>>,
    pub skeleton: Option<Skeleton>,
    pub state: Option<AnimationState>,
}

impl Renderer {
    pub fn default() -> Renderer {
        Renderer {
            asset: None,
            asset_animated: None,
            skeleton: None,
            state: None,
        }
    }
    pub fn set_asset(mut self, asset: Option<Arc<ModelAsset>>) -> Renderer {
        self.asset = asset;
        self.asset_animated = None;
        self
    }
    pub fn set_asset_animated(mut self, asset: Option<Arc<ModelAssetAnimated>>) -> Renderer {
        self.asset_animated = asset;
        self.asset = None;

        if let Some(asset_animated) = &self.asset_animated {
            let state_data = asset_animated.state_data.clone();
            let mut skel = Skeleton::new(asset_animated.skeleton_data.clone());
            skel.set_skin_by_name("goblin").unwrap();
            skel.set_to_setup_pose();
            self.skeleton = Some(skel);
            let mut s = AnimationState::new(state_data);
            let animation = asset_animated.skeleton_data.find_animation("walk").unwrap();
            s.set_animation(0, &animation, true);
            self.state = Some(s);
        }

        // let mut state = AnimationState::new(state_data);
        // let animation = skeleton_data.find_animation("walk").unwrap();
        // state.set_animation(0, &animation, true);

        self
    }
    pub fn generate_mesh(&mut self) -> Vec<Arc<Mesh>> {
        let mut out = Vec::new();

        let Some(skeleton) = &mut self.skeleton else {
            panic!();
        };
        let Some(state) = &mut self.state else {
            panic!();
        };

        let Some(asset_animated) = &mut self.asset_animated else {
            panic!();
        };

        // 5. Play an animation

        // Step/update once (simulate 1/60s frame)

        skeleton.update(1.0 / 60.0);
        state.update(1.0 / 60.0);
        state.apply(skeleton);
        skeleton.update_world_transform(rusty_spine::Physics::None);

        let mut z: f32 = 0.0;
        // iterate slots in draw order so attachments are in correct order
        for slot in skeleton.draw_order() {
            z += -0.1;
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

        out
    }
}
