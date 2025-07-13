#[derive(Clone)]
pub struct ColliderState {
    shape: ColliderShape,
    guid: i32,
}
impl ColliderState {
    pub fn new_box(guid: i32) -> ColliderState {
        ColliderState {
            shape: ColliderShape::Box(BoxColliderDef {}),
            guid: guid,
        }
    }
    pub fn new_sphere(guid: i32) -> ColliderState {
        ColliderState {
            shape: ColliderShape::Box(BoxColliderDef {}),
            guid: guid,
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

pub struct BoxColliderDef {}

#[derive(Clone)]
pub struct SphereColliderDef {}
#[derive(Clone)]

pub struct MeshColliderDef {}
