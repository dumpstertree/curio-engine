use crate::{cards::enums::card_events::CardEvents, state::host::state_card_attribute_modifier_stack::StateCardAttributeModifierStack};
use core::collections::game_state::GameState;

pub struct EventReciever {}
impl EventReciever {
    pub fn recieve(event: &CardEvents, game_state: &mut GameState) -> Vec<CardEvents> {
        match event {
            CardEvents::ClearModifiersAll() => {
                game_state.edit::<StateCardAttributeModifierStack>(|x| {
                    x.clear_all();
                });
            }
            _ => {}
        }
        vec![]
    }
}
