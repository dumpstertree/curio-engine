use std::collections::HashMap;

use wgpu::{Buffer, Device};

use crate::Collections::Mesh::Mesh;

pub struct Graphics_buffer_cache {
    v_buffer_hash: HashMap<i32, Buffer>,
    i_buffer_hash: HashMap<i32, Buffer>,
}
impl Graphics_buffer_cache {
    pub fn new() -> Graphics_buffer_cache {
        Graphics_buffer_cache {
            v_buffer_hash: HashMap::new(),
            i_buffer_hash: HashMap::new(),
        }
    }
    pub fn get_vertex_buffer(&mut self, device: &Device, mesh: &Mesh) -> (&Buffer, &Buffer) {
        let has = self.v_buffer_hash.contains_key(&mesh.instance_num);
        if !has {
            // self.insert(mesh.name.clone(), mesh.get_vertex_buffer_for_device(device));
            let id = mesh.instance_num.clone();
            let b = mesh.get_vertex_buffer_for_device(device);

            self.v_buffer_hash.insert(id, b);
        }
        let has2 = self.i_buffer_hash.contains_key(&mesh.instance_num);
        if !has2 {
            // self.insert(mesh.name.clone(), mesh.get_vertex_buffer_for_device(device));
            let id = mesh.instance_num.clone();
            let b = mesh.get_index_buffer_for_device(device);

            self.i_buffer_hash.insert(id, b);
        }
        (
            &self.v_buffer_hash[&mesh.instance_num.clone()],
            &self.i_buffer_hash[&mesh.instance_num.clone()],
        )
    }
    fn get_index_buffer(&mut self, device: &Device, mesh: &Mesh) -> &Buffer {
        let has = self.i_buffer_hash.contains_key(&mesh.instance_num);
        if !has {
            // self.insert(mesh.name.clone(), mesh.get_vertex_buffer_for_device(device));
            let id = mesh.instance_num.clone();
            let b = mesh.get_index_buffer_for_device(device);

            self.i_buffer_hash.insert(id, b);
        }
        &self.i_buffer_hash[&mesh.instance_num.clone()]
    }
}
