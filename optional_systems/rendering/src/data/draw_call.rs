use curio_core::{Color, Matrix4x4};
use std::{hash::Hash, sync::Arc};

use crate::{Material, Mesh};

#[derive(Clone, PartialEq)]
pub struct DrawCall {
    pub matrix: Vec<Matrix4x4>,
    pub mesh: Arc<Mesh>,
    pub materials: Arc<Material>,
    pub tint: Color,
    pub cast_shadow: bool,
}
impl Eq for DrawCall {}
impl Hash for DrawCall {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.matrix.hash(state);
        self.tint.hash(state);
        self.mesh.hash(state);
        self.materials.hash(state);
    }
}

impl DrawCall {
    pub fn draw_mesh_single<'a>(mesh: Arc<Mesh>, material: Arc<Material>, matrix: Matrix4x4, tint: Color, cast_shadow: bool) -> DrawCall {
        DrawCall {
            mesh: mesh,
            matrix: vec![matrix; 1],
            materials: material,
            tint,
            cast_shadow,
        }
    }
    pub fn draw_mesh_instanced<'a>(mesh: Arc<Mesh>, material: Arc<Material>, matrix: Vec<Matrix4x4>, tint: Color, cast_shadow: bool) -> DrawCall {
        DrawCall {
            mesh: mesh,
            matrix,
            materials: material,
            tint,
            cast_shadow,
        }
    }
}
