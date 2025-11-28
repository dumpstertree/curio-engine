use core::collections::state_ownerships::StateOwnerships;
use core::{collections::vector2_int::Vector2Int, system::system_game_state::IState};
use macro_state_serialize::global_state_serialize;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display, hash::Hash, sync::Arc};

#[derive(PartialEq, Eq)]
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
impl Hash for StateDeck {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut keys: Vec<&i32> = self.deck.keys().collect();
        keys.sort();
        keys.len().hash(state);
        for k in keys {
            k.hash(state);
            self.deck.get(k).unwrap().hash(state);
        }
    }
}
use rand::rng;
use rand::seq::SliceRandom;

use crate::cards::card_instance::CardInstance; // brings in the shuffle() method

pub enum CardLocation {
    Deck(i32),
    Discard(i32),
    Hand(i32),
}

#[derive(Hash, PartialEq, Eq, Default, Clone, Serialize, Deserialize)]
pub struct Deck {
    pub all_cards: Vec<Arc<CardInstance>>,
    pub pile_draw: Vec<Arc<CardInstance>>,
    pub pile_discard: Vec<Arc<CardInstance>>,
    pub hand_consumable: Vec<Arc<CardInstance>>,
    pub hand_persistent: Vec<Arc<CardInstance>>,
}
impl Deck {
    pub fn add_card_to_deck(&mut self, card_uid: &str, is_persistent: bool) {
        let inst = Arc::new(CardInstance::new(card_uid));

        println!("add card {} with id {}", inst.card_id, inst.instance_id);
        if is_persistent {
            self.hand_persistent.push(inst.clone());
        } else {
            self.pile_draw.push(inst.clone());
        }

        self.all_cards.push(inst);
    }
    pub fn get_instance(&self, instance_id: i32) -> Arc<CardInstance> {
        if let Some(pos) = self
            .all_cards
            .iter()
            .position(|x| x.instance_id == instance_id)
        {
            // Remove the item from `active`
            let item = self.all_cards.get(pos).unwrap();
            // Push it into `inactive`
            return item.clone();
        }

        println!("INSTANCE ID {}", instance_id);
        for x in &self.all_cards {
            println!("{}", x.instance_id);
        }

        panic!(" No card for {}", instance_id); // this is for some reason pulling from the other player
    }
    pub fn get_location(&self, card_instance: Arc<CardInstance>) -> CardLocation {
        if let Some(index) = self
            .pile_draw
            .iter()
            .position(|x| x.instance_id == card_instance.instance_id)
        {
            return CardLocation::Deck((self.pile_draw.len() - 1 - index) as i32);
        }
        if let Some(index) = self
            .pile_discard
            .iter()
            .position(|x| x.instance_id == card_instance.instance_id)
        {
            return CardLocation::Discard((self.pile_discard.len() - 1 - index) as i32);
        }
        if let Some(index) = self
            .hand_persistent
            .iter()
            .position(|x| x.instance_id == card_instance.instance_id)
        {
            return CardLocation::Hand(index as i32);
        }
        if let Some(index) = self
            .hand_consumable
            .iter()
            .position(|x| x.instance_id == card_instance.instance_id)
        {
            return CardLocation::Hand((self.hand_persistent.len() + index) as i32);
        }

        panic!();
    }
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
    pub fn discard(&mut self, card_instance: Arc<CardInstance>) {
        if let Some(pos) = self
            .hand_consumable
            .iter()
            .position(|x| x.card_id == card_instance.card_id)
        {
            // Remove the item from `active`
            let item = self.hand_consumable.remove(pos);
            // Push it into `inactive`
            self.pile_discard.push(item);
        }

        println!("card not found in hand consumable");
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
impl Display for CardTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CardTypes::Serve => write!(f, "SERVE"),
            CardTypes::Rest => write!(f, "REST"),
            CardTypes::Bump => write!(f, "BUMP"),
            CardTypes::Set => write!(f, "SET"),
            CardTypes::Spike => write!(f, "SPIKE"),
            CardTypes::Move => write!(f, "_"),
            CardTypes::Spell => write!(f, "SPELL"),
        }
    }
}
