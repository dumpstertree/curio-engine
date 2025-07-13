use crate::Collections::vector3::Vector3;

pub struct ComponentColliderBox {
    size: Vector3,
    guid: i32,
}

impl ComponentColliderBox {
    pub fn default() -> ComponentColliderBox {
        ComponentColliderBox {
            size: Vector3::one(),
            guid: 0,
        }
    }
    pub fn set_size(mut self, size: Vector3) -> ComponentColliderBox {
        self.size = size;
        self
    }
}
