use crate::Collections::vector3::Vector3;

pub struct Transform {
    pub position: Vector3,
}

impl Transform {
    pub fn default() -> Transform {
        Transform { position: Vector3::zero() }
    }
}
