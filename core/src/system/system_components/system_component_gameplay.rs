use crate::system::system_component::SystemComponent;
pub trait SystemComponentGameplay: SystemComponent {
    // fn set_systems(&mut self, systems: Vec<fn() -> Box<dyn ECSSystemEventless>>);
}
