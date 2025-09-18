use crate::cards::data_dep_empty::DataDepsEmpty;
use core::collections::vector2_int::Vector2Int;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result};

/// Data that has been filled and is ready to be passed into an attribute
#[derive(Clone, Serialize, Deserialize)]
pub enum DataDepsFilled {
    /// A list of Players. 0=all_ids
    Players(Vec<i32>),
    /// A list of Entities. 0=all_ids
    Entities(Vec<i32>),
    /// A list of Cards. 0=(player_id, card_id)
    Cards(Vec<i32>),
    /// A list of Tiles. 0=all_locations
    Tiles(Vec<Vector2Int>),
}

impl DataDepsFilled {
    /// check if the filled and empty data are the same content type
    pub fn is_aligned(&self, empty: &DataDepsEmpty) -> bool {
        match self {
            DataDepsFilled::Entities(_) => match empty {
                DataDepsEmpty::Entities(_) => return true,
                _ => return false,
            },
            DataDepsFilled::Cards(_) => match empty {
                DataDepsEmpty::Cards(_) => return true,
                _ => return false,
            },
            DataDepsFilled::Tiles(_) => match empty {
                DataDepsEmpty::Tiles(_) => return true,
                _ => return false,
            },
            DataDepsFilled::Players(_) => match empty {
                DataDepsEmpty::Players(_) => return true,
                _ => return false,
            },
        };
    }
    pub fn as_players(&self) -> Vec<i32> {
        match self {
            DataDepsFilled::Players(items) => items.clone(),
            _ => panic!("Tried to unwrap as 'Players' but was type {}", self),
        }
    }
    pub fn as_entities(&self) -> Vec<i32> {
        match self {
            DataDepsFilled::Entities(items) => items.clone(),
            _ => panic!("Tried to unwrap as 'Entities' but was type {}", self),
        }
    }
    pub fn as_cards(&self) -> Vec<i32> {
        match self {
            DataDepsFilled::Cards(items) => items.clone(),
            _ => panic!("Tried to unwrap as 'Cards' but was type {}", self),
        }
    }
    pub fn as_tiles(&self) -> Vec<Vector2Int> {
        match self {
            DataDepsFilled::Tiles(items) => items.clone(),
            _ => panic!("Tried to unwrap as 'Tiles' but was type {}", self),
        }
    }
}

impl Display for DataDepsFilled {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            DataDepsFilled::Cards(_) => return write!(f, "Cards"),
            DataDepsFilled::Tiles(_) => return write!(f, "Tiles"),
            DataDepsFilled::Players(_) => return write!(f, "Players"),
            DataDepsFilled::Entities(_) => return write!(f, "Entities"),
        }
    }
}
