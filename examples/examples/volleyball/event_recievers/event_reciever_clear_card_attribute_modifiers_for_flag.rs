use crate::{
    game_events::GameEvents,
    state::host::state_card_attribute_modifier_stack::StateCardAttributeModifierStack,
};
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::{ecs_event_reciever, ecs_system::ECSSystemEventless},
};
use ecs_event::global_ecs_system_event_reciever;
use ecs_system::global_ecs_system;
use hecs::World;

#[global_ecs_system]
pub struct EventRecieverApplyCardAttributeModifierRangeForEntities {}
impl ECSSystemEventless for EventRecieverApplyCardAttributeModifierRangeForEntities {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}
#[global_ecs_system_event_reciever(GameEvents)]
impl ecs_event_reciever::EventReciever<GameEvents> for EventRecieverApplyCardAttributeModifierRangeForEntities {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut World, _: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::ClearCardAttributeModifiersForFlag(clear_flag) => {
                game_state.edit::<StateCardAttributeModifierStack>(|x| {
                    x.clear_from_stack(*clear_flag);
                });
            }
            _ => {}
        }
    }
}
