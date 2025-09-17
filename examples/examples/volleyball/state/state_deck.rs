use core::{
    collections::{
        game_state::{GameState, StateOwnerships},
        vector2_int::Vector2Int,
    },
    system::system_game_state::IState,
};
use macro_state::global_state;
use macro_state_serialize::global_state_serialize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use winit::event;

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

use crate::{
    card_parser::{AttributeClearFlag, CardAttributes, CardEvents, TargetTypesCards, TargetTypesEntities, TargetTypesPlayers, TargetTypesTiles},
    game_events::GameEvents,
}; // brings in the shuffle() method

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
        for x in &self.pile_discard {
            self.pile_draw.push(x.clone());
        }

        // shuffle
        let mut rng = rng();
        self.pile_draw.shuffle(&mut rng);
    }
    pub fn draw(&mut self) {
        println!("draw");
        self.hand_consumable.push(self.pile_draw[0].clone());
        self.pile_draw.remove(0);
    }
}
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct CardInstance {
    pub card_id: String,
}

impl CardInstance {
    pub fn new(card_id: &str) -> CardInstance {
        CardInstance { card_id: String::from(card_id) }
    }
}

pub struct AttributeTargets {
    pub entities: Vec<i32>,
    pub cards: Vec<i32>,
    pub tile: Vec<Vector2Int>,
}
pub struct Card {
    pub title: String,
    pub card_type: CardTypes,
    pub cost: i32,
    pub model_path: String,
    // pub attributes: Vec<(fn(&GameState) -> AttributeTargets, AttributeClearFlag, CardAttributes)>,
    pub attributes: Vec<(AttributeClearFlag, CardAttributes)>,
    events: Vec<CardEvents>,
}
impl Card {
    pub fn new(title: &str, model_path: &str, card_type: CardTypes, cost: i32, attributes: Vec<(AttributeClearFlag, CardAttributes)>, events: Vec<CardEvents>) -> Card {
        Card {
            title: String::from(title),
            model_path: String::from(model_path),
            card_type,
            cost,
            attributes,
            events,
        }
    }
    pub fn get_events(&self) -> Vec<CardEvents> {
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

pub struct CardLibrary {
    cards: HashMap<&'static str, Arc<Card>>,
}
impl CardLibrary {
    pub fn new() -> CardLibrary {
        let mut hashmap: HashMap<&'static str, Arc<Card>> = HashMap::new();
        // rest
        hashmap.insert("rest", Arc::new(Card::new("rest", "card_bump.glb", CardTypes::Rest, 0, vec![], vec![])));

        // basic
        hashmap.insert("set+draw", Arc::new(Card::new("set+draw", "card_set.glb", CardTypes::Set, 1, vec![], vec![CardEvents::DrawCards(1, TargetTypesPlayers::User)])));
        hashmap.insert("set+move", Arc::new(Card::new("set+move", "card_set.glb", CardTypes::Set, 1, vec![], vec![CardEvents::MoveEntity(TargetTypesEntities::User, TargetTypesTiles::RandomOnTeamUser)])));
        hashmap.insert("bump", Arc::new(Card::new("bump", "card_bump.glb", CardTypes::Bump, 1, vec![], vec![CardEvents::MoveBallForward(1)])));
        hashmap.insert("spike", Arc::new(Card::new("spike", "card_spike.glb", CardTypes::Spike, 1, vec![], vec![CardEvents::MoveBallForward(2)])));

        // spells
        hashmap.insert("curse", Arc::new(Card::new("curse", "card_spike.glb", CardTypes::Spell, 1, vec![], vec![CardEvents::DiscardCards(TargetTypesCards::RandomOpponent)])));
        hashmap.insert("blessing", Arc::new(Card::new("blessing", "card_spike.glb", CardTypes::Spell, 1, vec![], vec![CardEvents::DrawCards(2, TargetTypesPlayers::User)])));
        hashmap.insert("deep_breath", Arc::new(Card::new("deep_breath", "card_spike.glb", CardTypes::Spell, 1, vec![], vec![CardEvents::GainEnergy(2, TargetTypesEntities::User)])));

        hashmap.insert(
            "hold_back",
            Arc::new(Card::new("hold_back", "card_spike.glb", CardTypes::Spell, 1, vec![(AttributeClearFlag::Play, CardAttributes::EditPlaysRange(-1, TargetTypesPlayers::User))], vec![])),
        );
        hashmap.insert(
            "extra_oomph",
            Arc::new(Card::new("extra_oomph", "card_spike.glb", CardTypes::Spell, 1, vec![(AttributeClearFlag::Play, CardAttributes::EditPlaysRange(1, TargetTypesPlayers::User))], vec![])),
        );

        CardLibrary { cards: hashmap }
    }
    pub fn get_card(&self, card_id: &str) -> Arc<Card> {
        self.cards[card_id].clone()
    }
}
