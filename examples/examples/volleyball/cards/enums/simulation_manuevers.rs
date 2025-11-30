use std::{fmt::Display, sync::Arc};

use crate::{
    cards::{card_dependencies::filled_card_response::FilledCardResponse, card_instance::CardInstance},
    game_board::Directions,
};

#[derive(Clone, Debug, Default)]
pub enum SimulationManuevers {
    #[default]
    Invalid,
    PlayCard(Arc<CardInstance>, FilledCardResponse),
    MoveEntity(Directions),
    EndTurn,
}
impl Display for SimulationManuevers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SimulationManuevers::PlayCard(x, _) => f.write_str(&format!("Play Card {}", x.card_id)),
            SimulationManuevers::MoveEntity(x) => f.write_str(&format!("Move Entity {}", x)),
            SimulationManuevers::EndTurn => f.write_str("End Turn"),
            SimulationManuevers::Invalid => f.write_str("Invalid"),
        }
    }
}
