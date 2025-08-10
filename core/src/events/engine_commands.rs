use crate::collections::vector3::Vector3;

#[derive(Clone)]
pub enum EngineCommands {
    Redraw,
    Tick,
    Exit,
    Resize(Vector3),
    Fullscreen(bool),
    Resizable(bool),
    Cursor(bool),
    SetDebugMode(bool),
    SetPauseMode(bool),
}
