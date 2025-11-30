use crate::{
    cards::{
        card_attributes::{card_attribute_events::CardAttributeEvents, card_attribute_modifier::CardAttributeModifiers, card_attribute_requirement::CardAttributeRequirement},
        card_library::CardLibrary,
        card_master::CardMaster,
        card_statement::CardStatement,
    },
    state::state_deck::CardTypes,
};
use core::{collections::game_state::GameState, random::Random};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize, Hash, Eq)]
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
    // get static values
    pub fn get_manuever_type(&self) -> CardTypes {
        self.get_master().card_type.clone()
    }
    pub fn get_title(&self) -> String {
        self.get_master().title.clone()
    }
    pub fn get_description(&self) -> String {
        self.get_master().description.clone()
    }

    // get values provided by statement
    pub fn get_cost(&self, game_state: &GameState, user_id: i32) -> i32 {
        self.get_statement(game_state, user_id).cost
    }
    pub fn get_attributes_events(&self, game_state: &GameState, user_id: i32) -> Vec<CardAttributeEvents> {
        self.get_statement(game_state, user_id).events.clone()
    }
    pub fn get_attributes_modifiers(&self, game_state: &GameState, user_id: i32) -> Vec<CardAttributeModifiers> {
        self.get_statement(game_state, user_id).attributes.clone()
    }
    pub fn get_attributes_requirements(&self, game_state: &GameState, user_id: i32) -> Vec<CardAttributeRequirement> {
        self.get_statement(game_state, user_id).requirements.clone()
    }

    // get the master
    pub fn get_master(&self) -> Arc<CardMaster> {
        CardLibrary::get_master_card(&self.card_id)
    }

    /// Checks the CardMaster 'statements' for the first one that matches based on state state of the game using the passed in GameState.
    /// If a statement has no requirements thats always considered a pass
    /// If none match the last element is returned
    pub fn get_statement(&self, game_state: &GameState, user_id: i32) -> CardStatement {
        let master = self.get_master();
        // println!("num statements : {}", master.statements.len());
        for statement in &master.statements {
            let is_met = statement
                .requirements
                .iter()
                .all(|x| x.is_met(&game_state, user_id));
            if is_met {
                return statement.clone();
            }
        }

        master.statements.last().unwrap().clone()
    }
    pub fn has_statement(&self, game_state: &GameState, user_id: i32) -> bool {
        let master = self.get_master();
        for statement in &master.statements {
            let is_met = statement
                .requirements
                .iter()
                .all(|x| x.is_met(&game_state, user_id));
            if is_met {
                return true;
            }
        }

        return false;
    }
}
