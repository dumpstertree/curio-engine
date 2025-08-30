use core::{collections::game_state::StateOwnerships, system::system_game_state::IState};
use macro_state_serialize::global_state_serialize;
use serde::{Deserialize, Serialize};

#[global_state_serialize]
pub struct StateDeck {
    pub deck: Deck,
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
use rand::seq::SliceRandom; // brings in the shuffle() method

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Deck {
    pub pile_draw: Vec<Card>,
    pub pile_discard: Vec<Card>,
    pub hand_consumable: Vec<Card>,
    pub hand_persistent: Vec<Card>,
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
        for _ in 0..5 {
            println!("draw");
            self.hand_consumable.push(self.pile_draw[0].clone());
            self.pile_draw.remove(0);
        }
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Card {
    pub title: String,
    pub card_type: CardTypes,
    pub cost: i32,
}
impl Card {
    pub fn new(title: &str, card_type: CardTypes, cost: i32) -> Card {
        Card { title: String::from(title), card_type, cost }
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub enum CardTypes {
    #[default]
    Serve,
    Rest,
    Bump,
    Set,
    Spike,
    Move,
}
