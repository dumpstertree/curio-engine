use crate::collections::{color::Color, matrix4x4::Matrix4x4, mesh::Mesh, vector3::Vector3};

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Gizmo {
    pub matrix: Vec<Matrix4x4>,
    pub mesh: Mesh,
    pub color: Color,
}

impl Gizmo {
    pub fn plane(matrix: Matrix4x4, width: f32, height: f32, color: Color) -> Gizmo {
        // let c = AssetLoader::generate_plane();

        Gizmo {
            color: color,
            matrix: vec![matrix],
            mesh: Mesh::primitive_plane(width, height, 1, 1),
        }
    }
    pub fn cube(matrix: Matrix4x4, size: Vector3, color: Color) -> Gizmo {
        // let c = AssetLoader::get_cube();
        Gizmo {
            color: color,
            matrix: vec![matrix],
            mesh: Mesh::primitive_cube2(size),
        }
    }
    pub fn sphere(matrix: Matrix4x4, diameter: f32, color: Color) -> Gizmo {
        // let c = AssetLoader::generate_sphere();

        Gizmo {
            color: color,
            matrix: vec![matrix],
            mesh: Mesh::primitive_sphere(diameter, 10, 10),
        }
    }
}
