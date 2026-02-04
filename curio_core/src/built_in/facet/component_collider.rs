use std::hash::Hash;

use crate::{
    extensions::extensions_f32::ExtensionsF32,
    {Matrix4x4, Vector3},
};

#[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
pub struct CollisionSnapshot {
    pub collider_a: ColliderSnapshot,
    pub collider_b: ColliderSnapshot,
    pub contact: Contact,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
pub struct Contact {
    pub point: Vector3,
    pub normal_a: Vector3,
    pub normal_b: Vector3,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Hash, PartialEq, Eq)]
pub struct ColliderSnapshot {
    pub guid: i32,
    pub matrix: Matrix4x4,
    pub shape: ColliderShape,
}

impl ColliderSnapshot {
    pub fn new(guid: i32, matrix: Matrix4x4, shape: ColliderShape) -> ColliderSnapshot {
        ColliderSnapshot { shape: shape, guid: guid, matrix: matrix }
    }
    pub const fn default() -> ColliderSnapshot {
        ColliderSnapshot {
            shape: ColliderShape::Sphere(SphereColliderDef { diameter: 1.0 }),
            guid: 0,
            matrix: Matrix4x4::zero(),
        }
    }
}
#[derive(Clone, serde::Serialize, serde::Deserialize, Hash, Eq, PartialEq)]
pub enum ColliderShape {
    Box(BoxColliderDef),
    Sphere(SphereColliderDef),
    Mesh(MeshColliderDef),
}
#[derive(Clone, serde::Serialize, serde::Deserialize, Hash, Eq, PartialEq)]
pub struct BoxColliderDef {
    pub size: Vector3,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SphereColliderDef {
    pub diameter: f32,
}
impl Hash for SphereColliderDef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.diameter.hash(state);
    }
}
impl Eq for SphereColliderDef {}

#[derive(Clone, serde::Serialize, serde::Deserialize, Hash, Eq, PartialEq)]
pub struct MeshColliderDef {}
