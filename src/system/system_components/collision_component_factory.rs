use crate::system::system_components::{
    collision_component::ICollisionComponent,
    collision_components::collision_component_default::CollisionComponentDefault,
};

pub struct SystemComponentCollisionFactory {}
impl SystemComponentCollisionFactory {
    pub fn create() -> Box<dyn ICollisionComponent> {
        Box::new(CollisionComponentDefault::new())
    }
}
