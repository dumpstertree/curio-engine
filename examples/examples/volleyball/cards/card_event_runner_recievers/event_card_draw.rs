use crate::{cards::enums::card_events::CardEvents, state::state_deck::StateDeck};
use curio_core::collections::game_state::GameState;

pub struct EventReciever {}
impl EventReciever {
    pub fn recieve(event: &CardEvents, game_state: &mut GameState) -> Vec<CardEvents> {
        match event {
            CardEvents::EventCardDraw(wrapped_entities, count) => {
                // edit state
                game_state.edit::<StateDeck>(|y| {
                    // unwrap the entites
                    let entity_ids = wrapped_entities.as_entities();
                    for entity_id in &entity_ids {
                        // get the deck for entity
                        let Some(deck) = y.deck.get_mut(&entity_id) else {
                            println!("Unable to find 'Deck' for UID {}", entity_id);
                            continue;
                        };

                        //draw cards based on count
                        deck.draw(*count);
                    }
                });
            }
            _ => {}
        }

        vec![]
    }
}
