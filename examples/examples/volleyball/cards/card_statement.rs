use crate::{
    cards::card_attributes::{card_attribute_events::CardAttributeEvents, card_attribute_modifier::CardAttributeModifiers, card_attribute_requirement::CardAttributeRequirement},
    state::state_deck::CardAttributeLifecycle,
};

#[derive(Default, Clone)]
/// A state of the CardMaster that is determined by its requirements.
/// This is important if you want a card to act differently depending on the state of the game
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
