use crate::{cards::card_modifier::CardModifier, cards::enums::card_events::CardEvents, state::host::state_card_attribute_modifier_stack::StateCardAttributeModifierStack};
use curio_core::collections::ledger::Ledger;

pub struct EventReciever {}
impl EventReciever {
    pub fn recieve(event: &CardEvents, ledger: &mut Ledger) -> Vec<CardEvents> {
        match event {
            CardEvents::ModifierEnergyForEntities(wrapped_entities, clear_flag, count) => {
                let entity_uids = wrapped_entities.as_entities();

                ledger.edit::<StateCardAttributeModifierStack>(|x| {
                    x.add_to_stack(CardModifier {
                        clear_flag: *clear_flag,
                        applies_to_players: vec![],
                        applies_to_entities: entity_uids.clone(),
                        applies_to_cards: vec![],
                        range: 0,
                        cost: 0,
                        energy: *count,
                    });
                });
            }
            _ => {}
        }
        vec![]
    }
}
