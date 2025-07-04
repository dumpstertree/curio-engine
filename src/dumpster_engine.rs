use crate::system::system_components::gameplay_component_factory::SystemComponentGameplayFactory;
use crate::system::system_components::gameplay_components::gameplay_component_default::ECSSystem;
use crate::system::system_components::graphics_component_factory::SystemComponentGraphicsFactory;
use crate::system::system_components::input_component_factory::SystemComponentInputFactory;
use crate::system::system_components::time_component_factory::SystemComponentTimeFactory;
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
            SystemComponentTimeFactory::create(),
            SystemComponentInputFactory::create(),
            SystemComponentGraphicsFactory::create(),
            SystemComponentGameplayFactory::create(ecs_systems),
        ]);

        // run the window
        system_window.run();
    }
}
