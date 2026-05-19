use crate::Vector2;
use serde::{Deserialize, Serialize};

/// Stored data data representing the current and change in the state of an Axis since last poll
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AxisState {
    pub position: Vector2,
    pub delta: Vector2,
}

impl AxisState {
    pub fn update(&mut self, axis: Vector2) {
        self.delta = axis - self.position;
        self.position = axis;
    }
}
