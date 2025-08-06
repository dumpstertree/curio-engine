use crate::{
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
    system::system_components::{gameplay_component::IGameplayComponent, gameplay_components::gameplay_component_default::GameplayComponentDefault},
};

pub struct SystemComponentGameplayFactory {}
impl SystemComponentGameplayFactory {
    pub fn create<TGameEvents>(ecs_systems_eventless: Vec<Box<dyn ECSSystemEventless>>) -> Box<dyn IGameplayComponent>
    where
        TGameEvents: 'static + Clone,
    {
        Box::new(GameplayComponentDefault::<TGameEvents>::new(ecs_systems_eventless))
    }
}
