use crate::{
    cards::{card_attribute_events::CardAttributeEvents, card_attribute_modifier::CardAttributeModifiers},
    state::state_deck::CardTypes,
};

pub struct CardMaster {
    pub title: String,
    pub card_type: CardTypes,
    pub cost: i32,
    pub model_path: String,
    // pub attributes: Vec<(fn(&GameState) -> AttributeTargets, AttributeClearFlag, CardAttributes)>,
    pub attributes: Vec<CardAttributeModifiers>,
    events: Vec<CardAttributeEvents>,
}
impl CardMaster {
    pub fn new(title: &str, model_path: &str, card_type: CardTypes, cost: i32, attributes: Vec<CardAttributeModifiers>, events: Vec<CardAttributeEvents>) -> CardMaster {
        CardMaster {
            title: String::from(title),
            model_path: String::from(model_path),
            card_type,
            cost,
            attributes,
            events,
        }
    }
    pub fn get_events(&self) -> Vec<CardAttributeEvents> {
        self.events.clone()
    }
    // pub fn new(title: &str, model_path: &str, card_type: CardTypes, cost: i32, attributes: Vec<(fn(&GameState) -> AttributeTargets, AttributeClearFlag, CardAttributes)>, events: Vec<CardEvents>) -> Card {
    //     Card {
    //         title: String::from(title),
    //         model_path: String::from(model_path),
    //         card_type,
    //         cost,
    //         attributes,
    //         events,
    //     }
    // }
}
