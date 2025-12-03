use crate::state::state_deck::Deck;

pub struct DeckLibrary {}

impl DeckLibrary {
    pub fn get_player_deck_standard() -> Deck {
        // create the empty deck
        let mut deck = Deck::default();

        // persistent
        deck.add_card_to_deck("heat", true);
        deck.add_card_to_deck("rest", true);
        deck.add_card_to_deck("serve", true);

        // manuevers
        deck.add_card_to_deck("wild_card", false);
        deck.add_card_to_deck("wild_card", false);
        deck.add_card_to_deck("wild_card", false);

        deck.add_card_to_deck("counter_spike", false);
        deck.add_card_to_deck("counter_spike", false);
        deck.add_card_to_deck("counter_spike", false);

        deck.add_card_to_deck("bump", false);
        deck.add_card_to_deck("bump", false);
        deck.add_card_to_deck("bump", false);
        deck.add_card_to_deck("spike", false);
        deck.add_card_to_deck("spike", false);
        deck.add_card_to_deck("set", false);
        deck.add_card_to_deck("set", false);
        deck.add_card_to_deck("set", false);

        // spells
        deck.add_card_to_deck("extra_oomph", false);
        deck.add_card_to_deck("hold_back", false);
        deck.add_card_to_deck("blessing", false);
        deck.add_card_to_deck("deep_breath", false);

        //return
        deck
    }
    pub fn get_ai_wild_deck() -> Deck {
        // create the empty deck
        let mut deck = Deck::default();

        // persistent
        deck.add_card_to_deck("rest", true);
        deck.add_card_to_deck("serve", true);

        // manuevers
        deck.add_card_to_deck("bump", false);
        deck.add_card_to_deck("bump", false);
        deck.add_card_to_deck("wild_card", false);
        deck.add_card_to_deck("wild_card", false);
        deck.add_card_to_deck("wild_card", false);
        deck.add_card_to_deck("bump", false);
        deck.add_card_to_deck("bump", false);
        deck.add_card_to_deck("wild_card", false);
        deck.add_card_to_deck("wild_card", false);
        deck.add_card_to_deck("wild_card", false);

        // spells
        deck.add_card_to_deck("blessing", false);
        deck.add_card_to_deck("deep_breath", false);
        // deck.add_card_to_deck("blessing", false);
        // deck.add_card_to_deck("deep_breath", false);

        deck
    }
    pub fn get_deck_for_uid(uid: &str) -> Deck {
        if uid == "wild" {
            //
            Self::get_ai_wild_deck()
        } else {
            Self::get_player_deck_standard()
        }
    }
}
