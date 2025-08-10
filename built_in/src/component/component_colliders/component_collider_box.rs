use core::{
    collections::vector3::Vector3,
    gameplay::ecs::component::component_collider::{BoxColliderDef, ColliderShape, CollisionSnapshot},
    random::Random,
};

pub struct ComponentColliderBox {
    pub size: Vector3,
    pub guid: i32,
    pub collisions: Vec<CollisionSnapshot>,
}

impl ComponentColliderBox {
    pub fn default() -> ComponentColliderBox {
        ComponentColliderBox {
            size: Vector3::one(),
            guid: Random::range_int(-9999, 9999),
            collisions: Vec::new(),
        }
    }
    pub fn set_size(mut self, size: Vector3) -> ComponentColliderBox {
        self.size = size;
        self
    }
    pub fn get_shape(&self) -> ColliderShape {
        ColliderShape::Box(BoxColliderDef { size: self.size })
    }
    pub fn is_colliding(&self) -> bool {
        self.collisions.len() > 0
    }
}
