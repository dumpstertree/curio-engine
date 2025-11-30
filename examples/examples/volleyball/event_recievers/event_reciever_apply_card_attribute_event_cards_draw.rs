use crate::{ai_resolver::CardEvents, state::state_deck::StateDeck};
use core::collections::game_state::GameState;

pub struct EventReciever {}
impl EventReciever {
    pub fn recieve(event: &CardEvents, game_state: &mut GameState) -> Vec<CardEvents> {
        match event {
            CardEvents::ApplyEventDrawCards(wrapped_entities, count) => {
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
                        for _ in 0..*count {
                            deck.draw();
                        }
                    }
                });
            }
            _ => {}
        }

        vec![]
    }
}
