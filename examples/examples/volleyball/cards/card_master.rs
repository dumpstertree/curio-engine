use crate::{
    cards::{card_attribute_events::CardAttributeEvents, card_attribute_modifier::CardAttributeModifiers, card_attribute_requirement::CardAttributeRequirement},
    state::state_deck::CardTypes,
};

pub struct CardMaster {
    pub title: String,
    pub card_type: CardTypes,
    pub cost: i32,
    pub model_path: String,
    pub description: String,
    // pub attributes: Vec<(fn(&GameState) -> AttributeTargets, AttributeClearFlag, CardAttributes)>,
    pub attributes: Vec<CardAttributeModifiers>,
    events: Vec<CardAttributeEvents>,
    pub requirements: Vec<CardAttributeRequirement>,
}
impl CardMaster {
    pub fn new(title: &str, model_path: &str, card_type: CardTypes, cost: i32, description: String, attributes: Vec<CardAttributeModifiers>, events: Vec<CardAttributeEvents>, requirements: Vec<CardAttributeRequirement>) -> CardMaster {
        CardMaster {
            title: String::from(title),
            model_path: String::from(model_path),
            card_type,
            cost,
            description,
            attributes,
            events,
            requirements,
        }
    }
    pub fn get_events(&self) -> Vec<CardAttributeEvents> {
        self.events.clone()
    }
}
