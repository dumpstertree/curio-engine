use core::{
    collections::{
        game_state::StateOwnerships,
        vector2_int::Vector2Int,
    },
    system::system_game_state::IState,
};
use macro_state_serialize::global_state_serialize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[global_state_serialize]
pub struct StateDeck {
    pub deck: HashMap<i32, Deck>,
}
impl IState for StateDeck {
    fn id() -> i32 {
        0007
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Host
    }
}
use rand::rng;
use rand::seq::SliceRandom;

use crate::cards::card_instance::CardInstance; // brings in the shuffle() method

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Deck {
    pub pile_draw: Vec<CardInstance>,
    pub pile_discard: Vec<CardInstance>,
    pub hand_consumable: Vec<CardInstance>,
    pub hand_persistent: Vec<CardInstance>,
}
impl Deck {
    pub fn reshuffle(&mut self) {
        println!("shuffle");
        for x in &self.hand_consumable {
            self.pile_discard.push(x.clone());
        }
        self.hand_consumable.clear();
        for x in &self.pile_discard {
            self.pile_draw.push(x.clone());
        }
        self.pile_discard.clear();

        // shuffle
        let mut rng = rng();
        self.pile_draw.shuffle(&mut rng);
    }
    pub fn draw(&mut self) {
        if self.pile_draw.len() == 0 {
            println!("Shuffled discard into draw");
            for x in &self.pile_discard {
                self.pile_draw.push(x.clone());
            }
            self.pile_discard.clear();
        }

        if self.pile_draw.len() == 0 {
            println!("No cards in draw or discard");
            return;
        }
        println!("draw");
        self.hand_consumable.push(self.pile_draw[0].clone());
        self.pile_draw.remove(0);
    }
}

pub struct AttributeTargets {
    pub entities: Vec<i32>,
    pub cards: Vec<i32>,
    pub tile: Vec<Vector2Int>,
}

#[derive(PartialEq, Eq, Default, Clone, Serialize, Deserialize)]
pub enum CardTypes {
    #[default]
    Serve,
    Rest,
    Bump,
    Set,
    Spike,
    Move,
    Spell,
}
