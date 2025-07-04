use crate::system::system_components::{input_component::input_component, input_components::input_component_default::InputComponentDefault};

pub struct SystemComponentInputFactory {}
impl SystemComponentInputFactory {
    pub fn create() -> Box<dyn input_component> {
        Box::new(InputComponentDefault::new())
    }
}
