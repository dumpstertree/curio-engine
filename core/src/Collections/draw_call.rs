use std::sync::Arc;

use crate::collections::{material::Material, matrix4x4::Matrix4x4, mesh::Mesh};

#[derive(Clone)]
pub struct DrawCall {
    pub matrix: Vec<Matrix4x4>,
    pub mesh: Vec<Arc<Mesh>>,
    pub materials: Vec<Arc<Material>>,
}

impl DrawCall {
    pub fn draw_mesh_single<'a>(mesh: Arc<Mesh>, material: Arc<Material>, matrix: Matrix4x4) -> DrawCall {
        DrawCall {
            mesh: vec![mesh; 1],
            matrix: vec![matrix; 1],
            materials: vec![material; 1],
        }
    }
    pub fn draw_mesh_instanced<'a>(mesh: Arc<Mesh>, material: Arc<Material>, matrix: Vec<Matrix4x4>) -> DrawCall {
        DrawCall {
            mesh: vec![mesh; 1],
            matrix,
            materials: vec![material; 1],
        }
    }
}
