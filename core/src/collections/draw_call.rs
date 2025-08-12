use crate::collections::{material::Material, matrix4x4::Matrix4x4, mesh::Mesh};

#[derive(Clone)]
pub struct DrawCall {
    pub matrix: Vec<Matrix4x4>,
    pub mesh: Vec<Mesh>,
    pub materials: Vec<Material>,
}

impl DrawCall {
    pub fn draw_mesh_single<'a>(mesh: Mesh, material: Material, matrix: Matrix4x4) -> DrawCall {
        DrawCall {
            mesh: vec![mesh; 1],
            matrix: vec![matrix; 1],
            materials: vec![material; 1],
        }
    }
    pub fn draw_mesh_instanced<'a>(mesh: Mesh, material: Material, matrix: Vec<Matrix4x4>) -> DrawCall {
        DrawCall {
            mesh: vec![mesh; 1],
            matrix,
            materials: vec![material; 1],
        }
    }
}
