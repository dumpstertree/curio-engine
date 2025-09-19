use crate::{
    game_events::GameEvents,
    state::{host::state_card_attribute_modifier_stack::StateCardAttributeModifierStack, state_energy::StateEnergy},
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
pub struct EventReciever {}
impl ECSSystemEventless for EventReciever {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}
#[global_ecs_system_event_reciever(GameEvents)]
impl ecs_event_reciever::EventReciever<GameEvents> for EventReciever {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut World, _: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::ApplyCardAttributeEventRefillEnergy(entity_ids) => {
                let state_card_attribute_modifier_stack = game_state.get_value2::<StateCardAttributeModifierStack>();
                game_state.edit::<StateEnergy>(|y| {
                    for x in entity_ids {
                        let d = state_card_attribute_modifier_stack.get_flat_stack_for_entity(*x);
                        let Some(entity) = y.all_players.get_mut(x) else {
                            continue;
                        };

                        entity.0 = entity.1 + d.energy;
                    }
                });
            }
            _ => {}
        }
    }
}
