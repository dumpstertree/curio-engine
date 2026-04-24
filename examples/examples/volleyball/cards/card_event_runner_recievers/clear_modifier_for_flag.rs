use crate::{cards::enums::card_events::CardEvents, state::host::state_card_attribute_modifier_stack::StateCardAttributeModifierStack};
use curio_core::collections::ledger::Ledger;

pub struct EventReciever {}
impl EventReciever {
    pub fn recieve(event: &CardEvents, ledger: &mut Ledger) -> Vec<CardEvents> {
        match event {
            CardEvents::ClearModifiersForFlag(clear_flag) => {
                ledger.edit::<StateCardAttributeModifierStack>(|x| {
                    x.clear_from_stack(*clear_flag);
                });
            }
            _ => {}
        }
        vec![]
    }
}
