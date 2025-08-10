use crate::collections::{matrix4x4::Matrix4x4, vector3::Vector3};

#[derive(Clone)]
pub struct CollisionSnapshot {
    pub collider_a: ColliderSnapshot,
    pub collider_b: ColliderSnapshot,
    pub contact: Contact,
}

#[derive(Clone)]
pub struct Contact {
    pub point: Vector3,
    pub normal_a: Vector3,
    pub normal_b: Vector3,
}

#[derive(Clone)]
pub struct ColliderSnapshot {
    pub guid: i32,
    pub matrix: Matrix4x4,
    pub shape: ColliderShape,
}
impl ColliderSnapshot {
    pub fn new(guid: i32, matrix: Matrix4x4, shape: ColliderShape) -> ColliderSnapshot {
        ColliderSnapshot {
            shape: shape,
            guid: guid,
            matrix: matrix,
        }
    }
    pub const fn default() -> ColliderSnapshot {
        ColliderSnapshot {
            shape: ColliderShape::Sphere(SphereColliderDef { diameter: 1.0 }),
            guid: 0,
            matrix: Matrix4x4::default(),
        }
    }
}
#[derive(Clone)]
pub enum ColliderShape {
    Box(BoxColliderDef),
    Sphere(SphereColliderDef),
    Mesh(MeshColliderDef),
}
#[derive(Clone)]
pub struct BoxColliderDef {
    pub size: Vector3,
}

#[derive(Clone)]
pub struct SphereColliderDef {
    pub diameter: f32,
}

#[derive(Clone)]
pub struct MeshColliderDef {}
