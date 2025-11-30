use core::collections::{
    event_queue::{EventScope, IGameEvent},
    vector2_int::Vector2Int,
};
use macro_events::global_events;
use std::{
    fmt::{Display, Formatter, Result},
    sync::Arc,
};

use crate::{
    cards::{card_dependencies::filled_card_response::FilledCardResponse, card_instance::CardInstance},
    state::{state_ball_mode::BallModes, state_teams::Teams},
};

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
}

impl Display for GameEvents {
    fn fmt(&self, f: &mut Formatter) -> Result {
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
            GameEvents::RequestUseManeuverPersistent(_, _, _) => write!(f, "RequestUseManeuverPersistent"),
            GameEvents::RequestUseManeuverConsumable(_, _, _) => write!(f, "RequestUseManeuverConsumable"),
            GameEvents::DrawCard() => write!(f, "DrawCard"),
            GameEvents::DiscardCards() => write!(f, "DiscardCards"),
            GameEvents::MoveEntity(_, _) => write!(f, "MoveEntity"),
        }
    }
}
