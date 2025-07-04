use crate::system::system_components::{graphics_component::IGraphicsComponent, graphics_components::graphics_component_wgpu::WGPUGraphicsComponent};

pub struct SystemComponentGraphicsFactory {}
impl SystemComponentGraphicsFactory {
    pub fn create() -> Box<dyn IGraphicsComponent> {
        Box::new(WGPUGraphicsComponent::new())
    }
}
