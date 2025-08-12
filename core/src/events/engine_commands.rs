use crate::collections::vector3::Vector3;

#[derive(Clone)]
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
