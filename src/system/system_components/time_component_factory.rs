use crate::system::system_components::{time_component::time_component, time_components::time_component::TimeComponent};

pub struct SystemComponentTimeFactory {}
impl SystemComponentTimeFactory {
    pub fn create() -> Box<dyn time_component> {
        Box::new(TimeComponent::new())
    }
}
