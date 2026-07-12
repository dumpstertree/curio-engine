use crate::{cards::enums::card_events::CardEvents, state::state_energy::StateEnergy};
use curio_core::collections::ledger::Ledger;

pub struct EventReciever {}
impl EventReciever {
    pub fn recieve(event: &CardEvents, ledger: &mut Ledger) -> Vec<CardEvents> {
        match event {
            CardEvents::EventEnergyEdit(wrapped_entities, count) => {
                // edit the gamestate
                ledger.write::<StateEnergy>(|x| {
                    // get the ids from the wrapped data
                    let entity_ids = wrapped_entities.as_entities();
                    // iterate over each entity
                    for entity_id in entity_ids {
                        // get the energy to mutate
                        let Some(entity) = x.all_players.get_mut(&entity_id) else {
                            println!("Unable to find 'Energy' for UID {}", entity_id);
                            continue;
                        };

                        // add the energy + delta
                        entity.0 = entity.0 + count;
                    }
                });
            }
            _ => {}
        }
        vec![]
    }
}
