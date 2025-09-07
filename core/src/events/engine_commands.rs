use std::fmt::Display;

use crate::collections::{
    event_queue::{EventScope, IGameEvent},
    vector3::Vector3,
};
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

impl IGameEvent for EngineCommands {
    fn get_scope(&self) -> EventScope {
        EventScope::Instance
    }
}

impl Display for EngineCommands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EngineCommand")
    }
}
