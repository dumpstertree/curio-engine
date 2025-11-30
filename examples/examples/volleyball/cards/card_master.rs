use crate::{
    cards::card_attributes::{card_attribute_events::CardAttributeEvents, card_attribute_modifier::CardAttributeModifiers, card_attribute_requirement::CardAttributeRequirement},
    state::state_deck::CardTypes,
};

#[derive(Default, Clone)]
pub struct CardStatement {
    pub cost: i32,
    pub requirements: Vec<CardAttributeRequirement>,
    pub attributes: Vec<CardAttributeModifiers>,
    pub events: Vec<CardAttributeEvents>,
}
impl CardStatement {
    pub fn new(cost: i32, requirements: Vec<CardAttributeRequirement>, attributes: Vec<CardAttributeModifiers>, events: Vec<CardAttributeEvents>) -> CardStatement {
        CardStatement { cost, requirements, attributes, events }
    }
}
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
