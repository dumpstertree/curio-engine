use core::{collections::state_ownerships::StateOwnerships, random::Random, system::system_game_state::IState};
use macro_state_serialize::global_state_serialize;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(PartialEq, Eq, Hash)]
#[global_state_serialize]
pub struct StateShop {
    pub shop: Shop,
}
impl IState for StateShop {
    fn id() -> i32 {
        901830449
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Host
    }
}
#[derive(Default, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Shop {
    pub stock: Vec<Stock>,
}
impl Shop {
    pub fn new(stock: Vec<Stock>) -> Shop {
        Shop { stock }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Stock {
    pub instance_id: i32,
    pub cost: i32,
    pub item: StockItems,
    pub count: i32,
}
impl Stock {
    pub fn new(item: StockItems, cost: i32, count: i32) -> Stock {
        Stock {
            instance_id: Random::range_int(-99999, 99999),
            cost: cost,
            item: item,
            count: count,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum StockItems {
    /// i32: card_id
    Card(String),
    /// i32: relic_id
    Relic(i32),
}
