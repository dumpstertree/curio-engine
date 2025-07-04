use crate::system::system_components::graphics_components::graphics_component_wgpu::WGPUGraphicsComponent;
use crate::system::system_components::{
    gameplay_components::gameplay_component_default::{ECSSystem, GameplayComponentDefault},
    input_components::input_component_default::InputComponentDefault,
    time_components::time_component::TimeComponent,
};

use crate::Window::SystemWindow::SystemWindow;

pub struct DumpsterEngine {}
impl DumpsterEngine {
    pub fn run<TGameEvents>(ecs_systems: Vec<Box<dyn ECSSystem<TGameEvents>>>)
    where
        TGameEvents: 'static,
        TGameEvents: Clone,
    {
        // create systems
        let mut system_window = SystemWindow::new(vec![
            // get generic values from factories
            Box::new(InputComponentDefault::new()),
            Box::new(WGPUGraphicsComponent::new()),
            Box::new(TimeComponent::new()),
            //apply the ecs systems
            Box::new(GameplayComponentDefault::new(ecs_systems)),
        ]);

        // run the window
        system_window.run();
    }
}
