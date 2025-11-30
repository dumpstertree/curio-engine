use crate::{ai_resolver::CardEvents, cards::card_modifier::CardModifier, state::host::state_card_attribute_modifier_stack::StateCardAttributeModifierStack};
use core::collections::game_state::GameState;

pub struct EventReciever {}
impl EventReciever {
    pub fn recieve(event: &CardEvents, game_state: &mut GameState) -> Vec<CardEvents> {
        match event {
            CardEvents::ApplyModifierRangeForEntities(wrapped_entities, clear_flag, count) => {
                let entity_uids = wrapped_entities.as_entities();

                game_state.edit::<StateCardAttributeModifierStack>(|x| {
                    x.add_to_stack(CardModifier {
                        clear_flag: *clear_flag,
                        applies_to_players: vec![],
                        applies_to_entities: entity_uids.clone(),
                        applies_to_cards: vec![],
                        range: *count,
                        cost: 0,
                        energy: 0,
                    });
                });
            }
            _ => {}
        }
        vec![]
    }
}
