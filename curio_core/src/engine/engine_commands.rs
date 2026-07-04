use crate::Vector3;
use serde::{Deserialize, Serialize};

// Commands for Curios and Plugins to interact
#[derive(Clone, Serialize, Deserialize)]
pub enum CurioCommands {
    // loop
    Tick,
    // lifecycle
    Exit,
    // window mgmt
    Resize(Vector3),
    Fullscreen(bool),
    Resizable(bool),
    Cursor(bool),
}
