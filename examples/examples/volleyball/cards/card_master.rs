use crate::{
    cards::card_statement::CardStatement,
    state::state_deck::{CardAttributeLifecycle, CardTypes},
};

pub struct CardMaster {
    pub title: String,
    pub card_type: CardTypes,
    pub description: String,
    pub statements: Vec<CardStatement>,
    pub life: Vec<CardAttributeLifecycle>,
}
impl CardMaster {
    pub fn new(title: &str, description: &str, card_type: CardTypes, statements: Vec<CardStatement>, life: Vec<CardAttributeLifecycle>) -> CardMaster {
        CardMaster {
            title: String::from(title),
            description: String::from(description),
            card_type,
            statements,
            life,
        }
    }
}
