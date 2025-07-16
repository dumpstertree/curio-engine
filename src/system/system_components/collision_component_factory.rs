use crate::system::system_components::{
    collision_component::ICollisionComponent,
    collision_components::collision_component_default::{self, CollisionComponentDefault},
    graphics_component::IGraphicsComponent,
    graphics_components::graphics_component_wgpu::WGPUGraphicsComponent,
};

pub struct SystemComponentCollisionFactory {}
impl SystemComponentCollisionFactory {
    pub fn create() -> Box<dyn ICollisionComponent> {
        Box::new(CollisionComponentDefault::new())
    }
}
