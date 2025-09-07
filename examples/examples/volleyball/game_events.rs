use core::collections::event_queue::IGameEvent;
use macro_events::global_events;
use std::fmt::{self};

use crate::state::{state_deck::Card, state_teams::Teams};

#[global_events]
pub enum GameEvents {
    // to -> instance
    Begin,
    PointScored(Teams),
    TurnBegin(i32),
    TurnEnd(i32),
    PlayCard(i32, Card),
    ResetBoard(Teams),

    // to -> peer
    DidTurnEnd(i32),
    DidTurnBegin(i32),
    // to -> host
    RequestTurnEnd(i32),
    RequestMoveZPos(i32),
    RequestMoveZNeg(i32),
    RequestMoveXPos(i32),
    RequestMoveXNeg(i32),
    RequestUseManeuverPersistent(i32, i32),
    RequestUseManeuverConsumable(i32, i32),
}

impl IGameEvent for GameEvents {
    fn get_scope(&self) -> core::collections::event_queue::EventScope {
        match &self {
            // to -> instance
            GameEvents::Begin => core::collections::event_queue::EventScope::Instance,
            GameEvents::TurnEnd(_) => core::collections::event_queue::EventScope::Instance,
            GameEvents::TurnBegin(_) => core::collections::event_queue::EventScope::Instance,
            GameEvents::PlayCard(_, _) => core::collections::event_queue::EventScope::Instance,
            GameEvents::PointScored(_) => core::collections::event_queue::EventScope::Instance,
            GameEvents::ResetBoard(_) => core::collections::event_queue::EventScope::Instance,
            // to -> host
            GameEvents::RequestTurnEnd(_) => core::collections::event_queue::EventScope::ConnectedHost,
            GameEvents::RequestMoveZPos(_) => core::collections::event_queue::EventScope::ConnectedHost,
            GameEvents::RequestMoveZNeg(_) => core::collections::event_queue::EventScope::ConnectedHost,
            GameEvents::RequestMoveXPos(_) => core::collections::event_queue::EventScope::ConnectedHost,
            GameEvents::RequestMoveXNeg(_) => core::collections::event_queue::EventScope::ConnectedHost,
            GameEvents::RequestUseManeuverPersistent(_, _) => core::collections::event_queue::EventScope::ConnectedHost,
            GameEvents::RequestUseManeuverConsumable(_, _) => core::collections::event_queue::EventScope::ConnectedHost,
            // to -> peer
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
            GameEvents::PlayCard(_, _) => write!(f, "PlayCard"),
            GameEvents::RequestTurnEnd(_) => write!(f, "RequestTurnEnd"),
            GameEvents::DidTurnEnd(_) => write!(f, "DidTurnEnd"),
            GameEvents::DidTurnBegin(_) => write!(f, "DidTurnBegin"),
            GameEvents::RequestMoveZPos(_) => write!(f, "RequestMoveZPos"),
            GameEvents::RequestMoveZNeg(_) => write!(f, "RequestMoveZNeg"),
            GameEvents::RequestMoveXPos(_) => write!(f, "RequestMoveXPos"),
            GameEvents::RequestMoveXNeg(_) => write!(f, "RequestMoveXNeg"),
            GameEvents::RequestUseManeuverPersistent(_, _) => write!(f, "RequestUseManeuverPersistent"),
            GameEvents::RequestUseManeuverConsumable(_, _) => write!(f, "RequestUseManeuverConsumable"),
        }
    }
}
