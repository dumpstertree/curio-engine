use crate::system::system_components::{
    gameplay_component::IGameplayComponent,
    gameplay_components::gameplay_component_default::{ECSSystem, ECSSystemEventless, GameplayComponentDefault},
};

pub struct SystemComponentGameplayFactory {}
impl SystemComponentGameplayFactory {
    pub fn create<TGameEvents>(
        ecs_systems: Vec<Box<dyn ECSSystem<TGameEvents>>>,
        ecs_systems_eventless: Vec<Box<dyn ECSSystemEventless>>,
    ) -> Box<dyn IGameplayComponent>
    where
        TGameEvents: Clone,
        TGameEvents: 'static,
    {
        Box::new(GameplayComponentDefault::new(ecs_systems, ecs_systems_eventless))
    }
}
