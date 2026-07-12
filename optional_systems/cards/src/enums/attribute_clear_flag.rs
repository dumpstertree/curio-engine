use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum ModifierClearFlag {
    Play,
    Turn,
    Game,
}
