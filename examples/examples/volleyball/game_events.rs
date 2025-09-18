use core::collections::{event_queue::IGameEvent, vector2_int::Vector2Int};
use macro_events::global_events;
use serde::{Deserialize, Serialize};
use std::fmt::{self};

use crate::{
    cards::{card_instance::CardInstance, data_dep_filled::DataDepsFilled},
    state::state_teams::Teams,
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
    PlayCard(i32, CardInstance, FilledCardResponse),
    ResetBoard(Teams),
    DrawCard(),
    DiscardCards(),
    MoveEntity(Vec<i32>, Vector2Int),
    // to -> peer
    DidTurnEnd(i32),
    DidTurnBegin(i32),
    // to -> host
    RequestTurnEnd(i32),
    RequestMoveZPos(i32),
    RequestMoveZNeg(i32),
    RequestMoveXPos(i32),
    RequestMoveXNeg(i32),
    RequestUseManeuverPersistent(i32, CardInstance, FilledCardResponse),
    RequestUseManeuverConsumable(i32, CardInstance, FilledCardResponse),
}

impl IGameEvent for GameEvents {
    fn get_scope(&self) -> core::collections::event_queue::EventScope {
        match &self {
            GameEvents::DrawCard() => core::collections::event_queue::EventScope::Instance,
            GameEvents::DiscardCards() => core::collections::event_queue::EventScope::Instance,
            GameEvents::MoveEntity(_, _) => core::collections::event_queue::EventScope::Instance,
            GameEvents::Begin => core::collections::event_queue::EventScope::Instance,
            GameEvents::TurnEnd(_) => core::collections::event_queue::EventScope::Instance,
            GameEvents::TurnBegin(_) => core::collections::event_queue::EventScope::Instance,
            GameEvents::PlayCard(_, _, _) => core::collections::event_queue::EventScope::Instance,
            GameEvents::PointScored(_) => core::collections::event_queue::EventScope::Instance,
            GameEvents::ResetBoard(_) => core::collections::event_queue::EventScope::Instance,
            GameEvents::RequestTurnEnd(_) => core::collections::event_queue::EventScope::ConnectedHost,
            GameEvents::RequestMoveZPos(_) => core::collections::event_queue::EventScope::ConnectedHost,
            GameEvents::RequestMoveZNeg(_) => core::collections::event_queue::EventScope::ConnectedHost,
            GameEvents::RequestMoveXPos(_) => core::collections::event_queue::EventScope::ConnectedHost,
            GameEvents::RequestMoveXNeg(_) => core::collections::event_queue::EventScope::ConnectedHost,
            GameEvents::RequestUseManeuverPersistent(_, _, _) => core::collections::event_queue::EventScope::ConnectedHost,
            GameEvents::RequestUseManeuverConsumable(_, _, _) => core::collections::event_queue::EventScope::ConnectedHost,
            GameEvents::DidTurnEnd(_) => core::collections::event_queue::EventScope::ConnectedPeers,
            GameEvents::DidTurnBegin(_) => core::collections::event_queue::EventScope::ConnectedPeers,
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
            GameEvents::RequestMoveXNeg(_) => write!(f, "RequestMoveXNeg"),
            GameEvents::RequestUseManeuverPersistent(_, _, _) => write!(f, "RequestUseManeuverPersistent"),
            GameEvents::RequestUseManeuverConsumable(_, _, _) => write!(f, "RequestUseManeuverConsumable"),
            GameEvents::DrawCard() => write!(f, "DrawCard"),
            GameEvents::DiscardCards() => write!(f, "DiscardCards"),
            GameEvents::MoveEntity(_, _) => write!(f, "MoveEntity"),
        }
    }
}
