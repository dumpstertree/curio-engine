use core::collections::{
    event_queue::{EventScope, IGameEvent},
    vector2_int::Vector2Int,
};
use macro_events::global_events;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, write},
    hash::Hash,
    sync::Arc,
};

use crate::{
    card_parser::AttributeClearFlag,
    cards::{card_dependencies::data_dep_filled::DataDepsFilled, card_instance::CardInstance},
    state::{state_ball_mode::BallModes, state_teams::Teams},
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FilledCardResponse {
    pub modifiers: Vec<FilledAttribute>,
    pub event: Vec<FilledAttribute>,
}
impl FilledCardResponse {
    pub fn new(state: Vec<FilledAttribute>, event: Vec<FilledAttribute>) -> FilledCardResponse {
        FilledCardResponse { modifiers: state, event }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FilledAttribute {
    pub filled: Vec<DataDepsFilled>,
}
impl FilledAttribute {
    pub fn new(filled: Vec<DataDepsFilled>) -> FilledAttribute {
        FilledAttribute { filled }
    }
}

#[global_events]
pub enum GameEvents {
    // an invalid default state
    #[default]
    Invalid,
    // to -> instance
    Begin,
    PointScored(Teams),
    TurnBegin(i32),
    TurnEnd(i32),
    PlayCard(i32, Arc<CardInstance>, FilledCardResponse),
    ResetBoard(Teams),
    DrawCard(),
    DiscardCards(),
    MoveEntity(Vec<i32>, Vector2Int),
    OnDidSetBallMode(BallModes),
    // to -> instance -> card events
    // ApplyCardAttributeEventRefillEnergy(Vec<i32>),
    // ApplyCardAttributeEventGainEnergy(Vec<i32>, i32),
    // ApplyCardAttributeEventDrawCards(Vec<i32>, i32),
    // ApplyCardAttributeEventDiscardCards(Vec<i32>),
    // ApplyCardAttributeEventMoveEntity(Vec<i32>, Vec<Vector2Int>),
    // ApplyCardAttributeEventMoveBall(Vec<Vector2Int>, i32, i32),
    // ApplyCardAttributeEventSetBallMode(BallModes),
    // // to -> instance -> card modifiers
    // ApplyCardAttributeModifierCostForEntities(AttributeClearFlag, Vec<i32>, i32),
    // ApplyCardAttributeModifierRangeForEntities(AttributeClearFlag, Vec<i32>, i32),
    // ApplyCardAttributeModifierEnergyForEntities(AttributeClearFlag, Vec<i32>, i32),
    // ClearCardAttributeModifiersForFlag(AttributeClearFlag),
    // ClearCardAttributeModifiersAll(),
    // to -> peer
    DidTurnEnd(i32),
    DidTurnBegin(i32),
    // to -> host
    RequestTurnEnd(i32),
    RequestMoveZPos(i32),
    RequestMoveZNeg(i32),
    RequestMoveXPos(i32),
    RequestMoveXNeg(i32),
    RequestUseManeuverPersistent(i32, i32, FilledCardResponse),
    RequestUseManeuverConsumable(i32, i32, FilledCardResponse),
}

impl IGameEvent for GameEvents {
    fn id() -> i32
    where
        Self: Sized + 'static,
    {
        0
    }
    fn ownership(&self) -> EventScope {
        match &self {
            GameEvents::Invalid => EventScope::Instance,
            GameEvents::DrawCard() => EventScope::Instance,
            GameEvents::DiscardCards() => EventScope::Instance,
            GameEvents::MoveEntity(_, _) => EventScope::Instance,
            GameEvents::Begin => EventScope::Instance,
            GameEvents::TurnEnd(_) => core::collections::event_queue::EventScope::Instance,
            GameEvents::TurnBegin(_) => EventScope::Instance,
            GameEvents::PlayCard(_, _, _) => EventScope::Instance,
            GameEvents::PointScored(_) => EventScope::Instance,
            GameEvents::ResetBoard(_) => EventScope::Instance,
            GameEvents::OnDidSetBallMode(_) => EventScope::Instance,
            // GameEvents::ApplyCardAttributeEventSetBallMode(_) => EventScope::Instance,
            // GameEvents::ApplyCardAttributeEventRefillEnergy(_) => EventScope::Instance,
            // GameEvents::ApplyCardAttributeEventGainEnergy(_, _) => EventScope::Instance,
            // GameEvents::ApplyCardAttributeEventMoveEntity(_, _) => EventScope::Instance,
            // GameEvents::ApplyCardAttributeEventDrawCards(_, _) => EventScope::Instance,
            // GameEvents::ApplyCardAttributeEventDiscardCards(_) => EventScope::Instance,
            // GameEvents::ApplyCardAttributeEventMoveBall(_, _, _) => EventScope::Instance,
            // GameEvents::ApplyCardAttributeModifierEnergyForEntities(_, _, _) => EventScope::Instance,
            // GameEvents::ApplyCardAttributeModifierCostForEntities(_, _, _) => EventScope::Instance,
            // GameEvents::ApplyCardAttributeModifierRangeForEntities(_, _, _) => EventScope::Instance,
            // GameEvents::ClearCardAttributeModifiersForFlag(attribute_clear_flag) => EventScope::Instance,
            // GameEvents::ClearCardAttributeModifiersAll() => EventScope::Instance,
            GameEvents::RequestTurnEnd(_) => EventScope::ConnectedHost,
            GameEvents::RequestMoveZPos(_) => EventScope::ConnectedHost,
            GameEvents::RequestMoveZNeg(_) => EventScope::ConnectedHost,
            GameEvents::RequestMoveXPos(_) => EventScope::ConnectedHost,
            GameEvents::RequestMoveXNeg(_) => EventScope::ConnectedHost,
            GameEvents::RequestUseManeuverPersistent(_, _, _) => EventScope::ConnectedHost,
            GameEvents::RequestUseManeuverConsumable(_, _, _) => EventScope::ConnectedHost,
            GameEvents::DidTurnEnd(_) => EventScope::All,
            GameEvents::DidTurnBegin(_) => EventScope::All,
        }
    }

    // fn id(&self) -> i32 {
    //     match self {
    //         GameEvents::Invalid => 1,
    //         GameEvents::Begin => 2,
    //         GameEvents::PointScored(teams) => 3,
    //         GameEvents::TurnBegin(_) => 4,
    //         GameEvents::TurnEnd(_) => 5,
    //         GameEvents::PlayCard(_, card_instance, filled_card_response) => 6,
    //         GameEvents::ResetBoard(teams) => 7,
    //         GameEvents::DrawCard() => 8,
    //         GameEvents::DiscardCards() => 9,
    //         GameEvents::MoveEntity(items, vector2_int) => 10,
    //         GameEvents::OnDidSetBallMode(ball_modes) => 11,
    //         GameEvents::ApplyCardAttributeEventRefillEnergy(items) => 12,
    //         GameEvents::ApplyCardAttributeEventGainEnergy(items, _) => 13,
    //         GameEvents::ApplyCardAttributeEventDrawCards(items, _) => 14,
    //         GameEvents::ApplyCardAttributeEventDiscardCards(items) => 15,
    //         GameEvents::ApplyCardAttributeEventMoveEntity(items, vector2_ints) => 16,
    //         GameEvents::ApplyCardAttributeEventMoveBall(vector2_ints, _, _) => 17,
    //         GameEvents::ApplyCardAttributeEventSetBallMode(ball_modes) => 18,
    //         GameEvents::ApplyCardAttributeModifierCostForEntities(attribute_clear_flag, items, _) => 19,
    //         GameEvents::ApplyCardAttributeModifierRangeForEntities(attribute_clear_flag, items, _) => 20,
    //         GameEvents::ApplyCardAttributeModifierEnergyForEntities(attribute_clear_flag, items, _) => 21,
    //         GameEvents::ClearCardAttributeModifiersForFlag(attribute_clear_flag) => 22,
    //         GameEvents::ClearCardAttributeModifiersAll() => 23,
    //         GameEvents::DidTurnEnd(_) => 24,
    //         GameEvents::DidTurnBegin(_) => 25,
    //         GameEvents::RequestTurnEnd(_) => 26,
    //         GameEvents::RequestMoveZPos(_) => 27,
    //         GameEvents::RequestMoveZNeg(_) => 28,
    //         GameEvents::RequestMoveXPos(_) => 29,
    //         GameEvents::RequestMoveXNeg(_) => 30,
    //         GameEvents::RequestUseManeuverPersistent(_, _, filled_card_response) => 31,
    //         GameEvents::RequestUseManeuverConsumable(_, _, filled_card_response) => 32,
    //     }
    // }
}

impl fmt::Display for GameEvents {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GameEvents::Invalid => write!(f, "Invalid"),
            GameEvents::Begin => write!(f, "Begin"),
            GameEvents::TurnBegin(_) => write!(f, "TurnBegin"),
            GameEvents::TurnEnd(_) => write!(f, "TurnEnd"),
            GameEvents::PointScored(_) => write!(f, "PointScored"),
            GameEvents::ResetBoard(_) => write!(f, "ResetBoard"),
            GameEvents::PlayCard(_, _, _) => write!(f, "PlayCard"),
            GameEvents::RequestTurnEnd(_) => write!(f, "RequestTurnEnd"),
            GameEvents::DidTurnEnd(_) => write!(f, "DidTurnEnd"),
            GameEvents::DidTurnBegin(_) => write!(f, "DidTurnBegin"),
            GameEvents::RequestMoveZPos(_) => write!(f, "RequestMoveZPos"),
            GameEvents::RequestMoveZNeg(_) => write!(f, "RequestMoveZNeg"),
            GameEvents::RequestMoveXPos(_) => write!(f, "RequestMoveXPos"),
            GameEvents::RequestMoveXNeg(_) => write!(f, "OnDidSetBallMode"),
            GameEvents::OnDidSetBallMode(_) => write!(f, "RequestMoveXNeg"),
            // GameEvents::ApplyCardAttributeEventSetBallMode(_) => write!(f, "SetBallMode"),
            GameEvents::RequestUseManeuverPersistent(_, _, _) => write!(f, "RequestUseManeuverPersistent"),
            GameEvents::RequestUseManeuverConsumable(_, _, _) => write!(f, "RequestUseManeuverConsumable"),
            GameEvents::DrawCard() => write!(f, "DrawCard"),
            GameEvents::DiscardCards() => write!(f, "DiscardCards"),
            GameEvents::MoveEntity(_, _) => write!(f, "MoveEntity"),
            // GameEvents::ApplyCardAttributeModifierEnergyForEntities(_, _, _) => write!(f, "ApplyCardAttributeModifierEnergyForEntities"),
            // GameEvents::ApplyCardAttributeModifierCostForEntities(_, _, _) => write!(f, "ApplyCardAttributeModifierCostForEntities"),
            // GameEvents::ApplyCardAttributeModifierRangeForEntities(_, _, _) => write!(f, "ApplyCardAttributeModifierRangeForEntities"),
            // GameEvents::ApplyCardAttributeEventRefillEnergy(_) => write!(f, "ApplyCardAttributeEventRefillEnergy"),
            // GameEvents::ApplyCardAttributeEventGainEnergy(_, _) => write!(f, "ApplyCardAttributeEventGainEnergy"),
            // GameEvents::ApplyCardAttributeEventMoveEntity(_, _) => write!(f, "ApplyCardAttributeEventMoveEntity"),
            // GameEvents::ApplyCardAttributeEventDrawCards(_, _) => write!(f, "ApplyCardAttributeEventDrawCards"),
            // GameEvents::ApplyCardAttributeEventDiscardCards(_) => write!(f, "ApplyCardAttributeEventDiscardCards"),
            // GameEvents::ApplyCardAttributeEventMoveBall(_, _, _) => write!(f, "ApplyCardAttributeEventMoveBall"),
            // GameEvents::ClearCardAttributeModifiersForFlag(_) => write!(f, "ClearCardAttributeModifiersForFlag"),
            // GameEvents::ClearCardAttributeModifiersAll() => write!(f, "ClearCardAttributeModifiersAll"),
        }
    }
}
