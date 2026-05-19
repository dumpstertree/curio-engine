use serde::{Deserialize, Serialize};

///Definitions for all types of range based inputs
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AxisCode {
    Cursor,
    Gamepad0StickLeft,
    Gamepad0StickRight,
    Gamepad0TriggerLeft,
    Gamepad0TriggerRight,
    Gamepad1StickLeft,
    Gamepad1StickRight,
    Gamepad1TriggerLeft,
    Gamepad1TriggerRight,
}
