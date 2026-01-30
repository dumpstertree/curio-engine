use std::fmt::Display;

use crate::collections::{event_queue::EventScope, vector3::Vector3};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum EngineCommands {
    Redraw,

    // tick
    Tick,
    // window
    Exit,
    Resize(Vector3),
    Fullscreen(bool),
    Resizable(bool),
    Cursor(bool),

    //editor
    SetDebugMode(bool),
    SetPauseMode(bool),

    // mulitplayer
    SetNumInputs(i32),
    SetNumScreens(i32),

    //
    SetHost(),
    SetPeer(),
}

impl EngineCommands {
    fn ownership(&self) -> EventScope {
        EventScope::Instance
    }

    fn id(&self) -> i32 {
        match self {
            EngineCommands::Redraw => 0,
            EngineCommands::Tick => 1,
            EngineCommands::Exit => 2,
            EngineCommands::Resize(_vector3) => 3,
            EngineCommands::Fullscreen(_) => 4,
            EngineCommands::Resizable(_) => 5,
            EngineCommands::Cursor(_) => 6,
            EngineCommands::SetDebugMode(_) => 7,
            EngineCommands::SetPauseMode(_) => 8,
            EngineCommands::SetNumInputs(_) => 9,
            EngineCommands::SetNumScreens(_) => 10,
            EngineCommands::SetHost() => 11,
            EngineCommands::SetPeer() => 12,
        }
    }
}

impl Display for EngineCommands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EngineCommand")
    }
}
