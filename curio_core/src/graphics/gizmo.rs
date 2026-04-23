use std::{hash::Hash, sync::Arc};

use crate::{graphics::mesh::Mesh, Color, Matrix4x4, Vector3};

#[derive(Clone, PartialEq)]
pub struct Gizmo {
    pub matrix: Vec<Matrix4x4>,
    pub mesh: Arc<Mesh>,
    pub color: Color,
}
impl Hash for Gizmo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.matrix.hash(state);
        self.mesh.hash(state);
        self.color.hash(state);
    }
}
impl Eq for Gizmo {}

impl Gizmo {
    pub fn plane(matrix: Matrix4x4, width: f32, height: f32, color: Color) -> Gizmo {
        // let c = AssetLoader::generate_plane();

        Gizmo {
            color: color,
            matrix: vec![matrix],
            mesh: Arc::new(Mesh::primitive_plane(width, height, 1, 1)),
        }
    }
    pub fn cube(matrix: Matrix4x4, size: Vector3, color: Color) -> Gizmo {
        // let c = AssetLoader::get_cube();
        Gizmo {
            color: color,
            matrix: vec![matrix],
            mesh: Arc::new(Mesh::primitive_cube2(size)),
        }
    }
    pub fn sphere(matrix: Matrix4x4, diameter: f32, color: Color) -> Gizmo {
        // let c = AssetLoader::generate_sphere();

        Gizmo {
            color: color,
            matrix: vec![matrix],
            mesh: Arc::new(Mesh::primitive_sphere(diameter, 10, 10)),
        }
    }
}
