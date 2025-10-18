use crate::{
    cards::{card_attribute_events::CardAttributeEvents, card_attribute_modifier::CardAttributeModifiers, card_attribute_requirement::CardAttributeRequirement},
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
    // pub cost: i32,
    // pub model_path: String,
    pub description: String,
    // pub attributes: Vec<(fn(&GameState) -> AttributeTargets, AttributeClearFlag, CardAttributes)>,
    // pub attributes: Vec<CardAttributeModifiers>,
    // events: Vec<CardAttributeEvents>,
    // pub requirements: Vec<CardAttributeRequirement>,
    pub statements: Vec<CardStatement>,
}
impl CardMaster {
    // pub fn new(title: &str, model_path: &str, card_type: CardTypes, cost: i32, description: String, attributes: Vec<CardAttributeModifiers>, events: Vec<CardAttributeEvents>, requirements: Vec<CardAttributeRequirement>) -> CardMaster {
    //     CardMaster {
    //         title: String::from(title),
    //         model_path: String::from(model_path),
    //         card_type,
    //         cost,
    //         description,
    //         attributes,
    //         events,
    //         requirements,
    //     }
    // }
    pub fn new(title: &str, description: &str, card_type: CardTypes, statements: Vec<CardStatement>) -> CardMaster {
        CardMaster {
            title: String::from(title),
            description: String::from(description),
            card_type,
            statements,
        }
    }
    // pub fn get_events(&self) -> Vec<CardAttributeEvents> {
    //     self.events.clone()
    // }
}
