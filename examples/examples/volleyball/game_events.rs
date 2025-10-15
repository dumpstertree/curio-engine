use core::collections::{
    event_queue::{EventScope, IGameEvent},
    vector2_int::Vector2Int,
};
use macro_events::global_events;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self},
    sync::Arc,
};

use crate::{
    card_parser::AttributeClearFlag,
    cards::{card_instance::CardInstance, data_dep_filled::DataDepsFilled},
    state::{state_ball_mode::BallModes, state_teams::Teams},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct FilledCardResponse {
    pub modifiers: Vec<Vec<DataDepsFilled>>,
    pub event: Vec<Vec<DataDepsFilled>>,
}
impl FilledCardResponse {
    pub fn new(state: Vec<Vec<DataDepsFilled>>, event: Vec<Vec<DataDepsFilled>>) -> FilledCardResponse {
        FilledCardResponse { modifiers: state, event }
    }
}

#[global_events]
pub enum GameEvents {
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
    ApplyCardAttributeEventRefillEnergy(Vec<i32>),
    ApplyCardAttributeEventGainEnergy(Vec<i32>, i32),
    ApplyCardAttributeEventMoveEntity(Vec<i32>, Vec<Vector2Int>),
    ApplyCardAttributeEventDrawCards(Vec<i32>, i32),
    ApplyCardAttributeEventDiscardCards(Vec<i32>),
    ApplyCardAttributeEventMoveBallForward(i32, i32, i32),
    ApplyCardAttributeEventMoveBallHorizontal(i32),
    // to -> instance -> card modifiers
    ApplyCardAttributeModifierCostForEntities(AttributeClearFlag, Vec<i32>, i32),
    ApplyCardAttributeModifierRangeForEntities(AttributeClearFlag, Vec<i32>, i32),
    ApplyCardAttributeModifierEnergyForEntities(AttributeClearFlag, Vec<i32>, i32),
    ClearCardAttributeModifiersForFlag(AttributeClearFlag),
    ClearCardAttributeModifiersAll(),
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
    fn get_scope(&self) -> EventScope {
        match &self {
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
            GameEvents::ApplyCardAttributeEventRefillEnergy(_) => EventScope::Instance,
            GameEvents::ApplyCardAttributeEventGainEnergy(_, _) => EventScope::Instance,
            GameEvents::ApplyCardAttributeEventMoveEntity(_, _) => EventScope::Instance,
            GameEvents::ApplyCardAttributeEventDrawCards(_, _) => EventScope::Instance,
            GameEvents::ApplyCardAttributeEventDiscardCards(_) => EventScope::Instance,
            GameEvents::ApplyCardAttributeEventMoveBallForward(_, _, _) => EventScope::Instance,
            GameEvents::ApplyCardAttributeEventMoveBallHorizontal(_) => EventScope::Instance,
            GameEvents::ApplyCardAttributeModifierEnergyForEntities(_, _, _) => EventScope::Instance,
            GameEvents::ApplyCardAttributeModifierCostForEntities(_, _, _) => EventScope::Instance,
            GameEvents::ApplyCardAttributeModifierRangeForEntities(_, _, _) => EventScope::Instance,
            GameEvents::ClearCardAttributeModifiersForFlag(attribute_clear_flag) => EventScope::Instance,
            GameEvents::ClearCardAttributeModifiersAll() => EventScope::Instance,
            GameEvents::RequestTurnEnd(_) => EventScope::ConnectedHost,
            GameEvents::RequestMoveZPos(_) => EventScope::ConnectedHost,
            GameEvents::RequestMoveZNeg(_) => EventScope::ConnectedHost,
            GameEvents::RequestMoveXPos(_) => EventScope::ConnectedHost,
            GameEvents::RequestMoveXNeg(_) => EventScope::ConnectedHost,
            GameEvents::RequestUseManeuverPersistent(_, _, _) => EventScope::ConnectedHost,
            GameEvents::RequestUseManeuverConsumable(_, _, _) => EventScope::ConnectedHost,
            GameEvents::DidTurnEnd(_) => EventScope::ConnectedPeers,
            GameEvents::DidTurnBegin(_) => EventScope::ConnectedPeers,
        }
    }
}

impl fmt::Display for GameEvents {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
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
            GameEvents::RequestUseManeuverPersistent(_, _, _) => write!(f, "RequestUseManeuverPersistent"),
            GameEvents::RequestUseManeuverConsumable(_, _, _) => write!(f, "RequestUseManeuverConsumable"),
            GameEvents::DrawCard() => write!(f, "DrawCard"),
            GameEvents::DiscardCards() => write!(f, "DiscardCards"),
            GameEvents::MoveEntity(_, _) => write!(f, "MoveEntity"),
            GameEvents::ApplyCardAttributeModifierEnergyForEntities(_, _, _) => todo!(),
            GameEvents::ApplyCardAttributeModifierCostForEntities(_, _, _) => todo!(),
            GameEvents::ApplyCardAttributeModifierRangeForEntities(_, _, _) => todo!(),
            GameEvents::ApplyCardAttributeEventRefillEnergy(_) => todo!(),
            GameEvents::ApplyCardAttributeEventGainEnergy(_, _) => todo!(),
            GameEvents::ApplyCardAttributeEventMoveEntity(_, _) => todo!(),
            GameEvents::ApplyCardAttributeEventDrawCards(_, _) => todo!(),
            GameEvents::ApplyCardAttributeEventDiscardCards(_) => todo!(),
            GameEvents::ApplyCardAttributeEventMoveBallForward(_, _, _) => todo!(),
            GameEvents::ApplyCardAttributeEventMoveBallHorizontal(_) => todo!(),
            GameEvents::ClearCardAttributeModifiersForFlag(_) => todo!(),
            GameEvents::ClearCardAttributeModifiersAll() => todo!(),
        }
    }
}
