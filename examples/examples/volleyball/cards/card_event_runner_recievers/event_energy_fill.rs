use crate::{
    cards::enums::card_events::CardEvents,
    state::{host::state_card_attribute_modifier_stack::StateCardAttributeModifierStack, state_energy::StateEnergy},
};
use curio_core::collections::ledger::Ledger;

pub struct EventReciever {}
impl EventReciever {
    pub fn recieve(event: &CardEvents, ledger: &mut Ledger) -> Vec<CardEvents> {
        match event {
            CardEvents::EventEnergyFill(wrapped_entities) => {
                // get modifiers
                let state_card_attribute_modifier_stack = ledger.get::<StateCardAttributeModifierStack>();

                // edit the energy
                ledger.edit::<StateEnergy>(|x| {
                    // unwrap entityes
                    let entity_ids = wrapped_entities.as_entities();
                    // iterate over each entity
                    for entity_id in &entity_ids {
                        // get the active modifiers
                        let modifiers = state_card_attribute_modifier_stack.get_flat_stack_for_entity(*entity_id);
                        // get the energy for the entity
                        let Some(entity) = x.all_players.get_mut(entity_id) else {
                            println!("Unable to find 'Energy' for UID {}", entity_id);
                            continue;
                        };

                        // set the energy to max energy + the max energy modifier
                        entity.0 = entity.1 + modifiers.energy;
                    }
                });
            }
            _ => {}
        }
        vec![]
    }
}
