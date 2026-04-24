use crate::{cards::enums::card_events::CardEvents, state::state_deck::StateDeck};
use curio_core::collections::ledger::Ledger;

pub struct EventReciever {}
impl EventReciever {
    pub fn recieve(event: &CardEvents, ledger: &mut Ledger) -> Vec<CardEvents> {
        match event {
            CardEvents::EventCardDiscard(wrapped_cards) => {
                // unwrap the cards
                let card_uids = wrapped_cards.as_cards();

                // edit the state
                ledger.edit::<StateDeck>(|x| {
                    // iterate over each entity and its deck
                    for uid_deck in x.deck.iter_mut() {
                        // pull out the deck we are editing
                        let deck = uid_deck.1;

                        // iterate over each card in each users deck backwards to remove uninterupted
                        for i in (0..deck.hand_consumable.len()).rev() {
                            let card = &deck.hand_consumable[i];

                            // check if this card is in the list of cards to discard
                            let is_in_deck = card_uids.contains(&card.instance_id);
                            if !is_in_deck {
                                continue;
                            }
                            // discard the card
                            deck.discard(card.clone());
                        }
                    }
                });
            }
            _ => {}
        }

        vec![]
    }
}
