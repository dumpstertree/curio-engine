use curio_core::collections::state_ownerships::StateOwnerships;
use curio_core::{collections::vector2_int::Vector2Int, system::system_game_state::IState};
use record_serializable::record_serializable;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display, hash::Hash, sync::Arc};

#[derive(PartialEq, Eq)]
#[record_serializable]
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
    OutOfPlay(i32),
}

#[derive(Hash, PartialEq, Eq, Default, Clone, Serialize, Deserialize)]
pub struct Deck {
    pub all_cards: Vec<Arc<CardInstance>>,
    pub pile_draw: Vec<Arc<CardInstance>>,
    pub pile_discard: Vec<Arc<CardInstance>>,
    pub pile_exhuast: Vec<Arc<CardInstance>>,
    pub pile_consume: Vec<Arc<CardInstance>>,
    pub pile_exile: Vec<Arc<CardInstance>>,
    pub hand_consumable: Vec<Arc<CardInstance>>,
    // pub hand_persistent: Vec<Arc<CardInstance>>,
}
impl Deck {
    // pub fn add_card_to_deck(&mut self, card_uid: &str, is_persistent: bool) {
    //     let inst = Arc::new(CardInstance::new(card_uid));

    //     println!("add card {} with id {}", inst.card_id, inst.instance_id);
    //     if is_persistent {
    //         self.hand_persistent.push(inst.clone());
    //     } else {
    //         self.pile_draw.push(inst.clone());
    //     }

    //     self.all_cards.push(inst);
    // }
    pub fn get_cards_from_all(&self, predicate: fn(&Arc<CardInstance>) -> bool) -> Vec<Arc<CardInstance>> {
        let mut cards = Vec::new();
        for card in &self.all_cards {
            if predicate(card) {
                cards.push(card.clone());
            }
        }
        cards
    }
    pub fn get_cards_from_hand(&self, predicate: fn(&Arc<CardInstance>) -> bool) -> Vec<Arc<CardInstance>> {
        let mut cards = Vec::new();
        for card in &self.hand_consumable {
            if predicate(card) {
                cards.push(card.clone());
            }
        }
        cards
    }
    pub fn hand_len(&self) -> i32 {
        let mut len = 0;
        for c in &self.hand_consumable {
            if c.get_attributes_lifecycle()
                .contains(&CardAttributeLifecycle::Light)
            {
                len = len + 0;
                continue;
            }
            if c.get_attributes_lifecycle()
                .contains(&CardAttributeLifecycle::Heavy)
            {
                len = len + 2;
                continue;
            }
            len = len + 1;
        }

        len
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

        // println!("INSTANCE ID {}", instance_id);
        // for x in &self.all_cards {
        //     println!("{}", x.instance_id);
        // }

        panic!(" No card for {}", instance_id); // this is for some reason pulling from the other player
    }
    pub fn get_location(&self, card_instance: Arc<CardInstance>, predicate: fn(&Arc<CardInstance>) -> bool) -> Option<CardLocation> {
        if let Some(index) = self
            .pile_draw
            .iter()
            .filter(|x| predicate(x))
            .position(|x| x.instance_id == card_instance.instance_id)
        {
            return Some(CardLocation::Deck((self.pile_draw.len() - 1 - index) as i32));
        }
        if let Some(index) = self
            .pile_discard
            .iter()
            .filter(|x| predicate(x))
            .position(|x| x.instance_id == card_instance.instance_id)
        {
            return Some(CardLocation::Discard((self.pile_discard.len() - 1 - index) as i32));
        }

        // if let Some(index) = self
        //     .hand_persistent
        //     .iter()
        //     .position(|x| x.instance_id == card_instance.instance_id)
        // {
        //     return Some(CardLocation::Hand(index as i32));
        // }
        if let Some(index) = self
            .hand_consumable
            .iter()
            .filter(|x| predicate(x))
            .position(|x| x.instance_id == card_instance.instance_id)
        {
            return Some(CardLocation::Hand((index) as i32));
            // return Some(CardLocation::Hand((self.hand_persistent.len() + index) as i32));
        }
        if let Some(index) = self
            .pile_exhuast
            .iter()
            .filter(|x| predicate(x))
            .position(|x| x.instance_id == card_instance.instance_id)
        {
            return Some(CardLocation::OutOfPlay((index) as i32));
        }
        if let Some(index) = self
            .pile_exile
            .iter()
            .filter(|x| predicate(x))
            .position(|x| x.instance_id == card_instance.instance_id)
        {
            return Some(CardLocation::OutOfPlay((index) as i32));
        }
        if let Some(index) = self
            .pile_consume
            .iter()
            .filter(|x| predicate(x))
            .position(|x| x.instance_id == card_instance.instance_id)
        {
            return Some(CardLocation::OutOfPlay((index) as i32));
        }

        None
    }
}
impl Deck {
    pub fn remove_card_from_deck(&mut self, instance_id: &i32) {
        self.all_cards.retain(|x| x.instance_id != *instance_id);
        self.pile_draw.retain(|x| x.instance_id != *instance_id);
        self.pile_consume.retain(|x| x.instance_id != *instance_id);
        self.pile_exile.retain(|x| x.instance_id != *instance_id);
        self.pile_exhuast.retain(|x| x.instance_id != *instance_id);
        self.hand_consumable
            .retain(|x| x.instance_id != *instance_id);
    }
    /// Move a card into deck. (unchanged semantics, but explicit)
    pub fn add_card_to_deck(&mut self, card_uid: &str, _is_persistent: bool) {
        let inst = Arc::new(CardInstance::new(card_uid));
        // println!("add card {} with id {}", inst.card_id, inst.instance_id);

        // if is_persistent {
        // self.hand_persistent.push(inst.clone());
        // } else {
        // self.pile_draw.push(inst.clone());
        // }

        self.pile_draw.push(inst.clone());
        self.all_cards.push(inst);
    }

    /// Reshuffle: move hand -> discard, discard -> draw, then shuffle draw.
    pub fn reshuffle(&mut self) {
        // Move consumable hand to discard without cloning
        self.pile_discard.extend(self.hand_consumable.drain(..));

        // Move discard into draw by appending (no clones, no extra allocations)
        self.pile_draw.append(&mut self.pile_discard);

        // Move discard into draw by appending (no clones, no extra allocations)
        self.pile_draw.append(&mut self.pile_exhuast);

        // shuffle
        let mut rng = rng();
        self.pile_draw.shuffle(&mut rng);

        let mut reliable_cards: Vec<(i32, Arc<CardInstance>)> = Vec::new();

        for i in (0..self.pile_draw.len()).rev() {
            let card = &self.pile_draw[i];

            // Look for a Reliable(x) attribute
            if let Some(CardAttributeLifecycle::Reliable(idx)) = card
                .get_attributes_lifecycle()
                .iter()
                .find(|a| matches!(a, CardAttributeLifecycle::Reliable(_)))
            {
                // Remove card from pile_draw, store with its target index
                let removed = self.pile_draw.remove(i);
                reliable_cards.push((*idx, removed));
            }
        }

        reliable_cards.sort_by_key(|(target_idx, _card)| *target_idx);

        for (target_idx, card) in reliable_cards {
            let idx_usize = target_idx.max(0) as usize;

            if idx_usize >= self.pile_draw.len() {
                // If index is past end, push
                self.pile_draw.push(card);
            } else {
                self.pile_draw.insert(idx_usize, card);
            }
        }
    }

    /// Draw one card into hand_consumable (respect hand size)
    pub fn draw(&mut self, count: i32) {
        let mut drawn = 0;
        while drawn < count {
            // if self.hand_consumable.len() >= 10 {
            //     return;
            // }

            if self.pile_draw.is_empty() {
                // Move discard into draw (no clones)
                self.pile_draw.append(&mut self.pile_discard);
                // we dont append exhuast
                // self.pile_draw.append(&mut self.pile_exhuast);

                let mut rng = rng();
                self.pile_draw.shuffle(&mut rng);
            }

            if self.pile_draw.is_empty() {
                // no cards to draw
                return;
            }

            let first_quick_index = self.pile_draw.iter().position(|x| {
                x.get_attributes_lifecycle()
                    .contains(&CardAttributeLifecycle::Quick)
            });

            let card;
            // if we have a quick card in the draw pile we draw that card
            if let Some(first_quick_index) = first_quick_index {
                // if you need to keep your draw order where index 0 is top, then:
                card = self.pile_draw.remove(first_quick_index);
            } else {
                // if you need to keep your draw order where index 0 is top, then:
                card = self.pile_draw.remove(0);
            }

            if !card
                .get_attributes_lifecycle()
                .contains(&CardAttributeLifecycle::Light)
            {
                drawn += 1;
            }

            self.hand_consumable.push(card);
        }
    }

    pub fn play(&mut self, card_instance: Arc<CardInstance>) {
        let lifecycle_attributes = card_instance.get_attributes_lifecycle();
        let is_persistant = lifecycle_attributes.contains(&CardAttributeLifecycle::Persistant);
        if is_persistant {
            return;
        }

        if let Some(pos) = self
            .hand_consumable
            .iter()
            .position(|c| c.instance_id == card_instance.instance_id)
        {
            // remove card
            let card = self.hand_consumable.remove(pos);

            // add card to consumed pile
            let is_consume = lifecycle_attributes.contains(&CardAttributeLifecycle::Consume);
            if is_consume {
                self.pile_consume.push(card);
                return;
            }

            // add card to exile pile
            let is_exile = lifecycle_attributes.contains(&CardAttributeLifecycle::Exile);
            if is_exile {
                self.pile_exile.push(card);
                return;
            }

            // add card to exhuast pile
            let is_exhuast = lifecycle_attributes.contains(&CardAttributeLifecycle::Exhuast);
            if is_exhuast {
                self.pile_exhuast.push(card);
                return;
            }

            // add card to discard
            self.pile_discard.push(card);
        } else {
            // not found — helpful debug / detect caller errors
            eprintln!("discard: card {} not found in hand_consumable (hand len {})", card_instance.instance_id, self.hand_consumable.len());
        }
    }
    /// Discard a card (move from hand_consumable to pile_discard)
    pub fn discard(&mut self, card_instance: Arc<CardInstance>) {
        let lifecycle_attributes = card_instance.get_attributes_lifecycle();
        let is_linger = lifecycle_attributes.contains(&CardAttributeLifecycle::Linger);

        // lifecycle_attributes.contains()

        if is_linger {
            return;
        }
        if let Some(pos) = self
            .hand_consumable
            .iter()
            .position(|c| c.instance_id == card_instance.instance_id)
        {
            let card = self.hand_consumable.remove(pos);

            // if we are not lingering we push to discard
            self.pile_discard.push(card);
        } else {
            // not found — helpful debug / detect caller errors
            eprintln!("discard: card {} not found in hand_consumable (hand len {})", card_instance.instance_id, self.hand_consumable.len());
        }
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
    Food,
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
            CardTypes::Food => write!(f, "FOOD"),
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
pub enum CardAttributeLifecycle {
    Quick,
    Exhuast,
    Exile,
    Consume,
    Linger,
    Light,
    Heavy,
    Persistant,
    Reliable(i32),
}
