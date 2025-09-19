use crate::{
    cards::{card_attribute_events::CardAttributeEvents, card_attribute_modifier::CardAttributeModifiers, card_library::CardLibrary, card_master::CardMaster},
    state::state_deck::CardTypes,
};
use core::random::Random;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(PartialEq, Default, Clone, Serialize, Deserialize)]
pub struct CardInstance {
    pub card_id: String,
    pub instance_id: i32,
}

impl CardInstance {
    pub fn new(card_id: &str) -> CardInstance {
        CardInstance {
            card_id: String::from(card_id),
            instance_id: Random::range_int(-9999, 9999),
        }
    }
}
impl CardInstance {
    pub fn get_visual_uid(&self) -> String {
        self.get_master().model_path.clone()
    }
    pub fn get_manuever_type(&self) -> CardTypes {
        self.get_master().card_type.clone()
    }
    pub fn get_title(&self) -> String {
        self.get_master().title.clone()
    }
    pub fn get_cost(&self) -> i32 {
        self.get_master().cost
    }
    pub fn get_attributes_events(&self) -> Vec<CardAttributeEvents> {
        self.get_master().get_events()
    }
    pub fn get_attributes_modifiers(&self) -> Vec<CardAttributeModifiers> {
        self.get_master().attributes.clone()
    }
    pub fn get_master(&self) -> Arc<CardMaster> {
        CardLibrary::get_master_card(&self.card_id)
    }
}
