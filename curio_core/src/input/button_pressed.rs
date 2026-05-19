use serde::{Deserialize, Serialize};

/// A enum representing if a button is down or up
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ButtonPressed {
    #[default]
    Down,
    Up,
}
