use crate::{cards::card_statement::CardStatement, state::state_deck::CardTypes};

pub struct CardMaster {
    pub title: String,
    pub card_type: CardTypes,
    pub description: String,
    pub statements: Vec<CardStatement>,
}
impl CardMaster {
    pub fn new(title: &str, description: &str, card_type: CardTypes, statements: Vec<CardStatement>) -> CardMaster {
        CardMaster {
            title: String::from(title),
            description: String::from(description),
            card_type,
            statements,
        }
    }
}
