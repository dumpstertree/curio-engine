use curio_core::{Vector2Int, collections::state_ownerships::StateOwnerships, system::system_game_state::IState};
use std::hash::Hash;

use record::record;

use crate::cards::{card_attributes_targets::attribute_target_type_tiles::AttributeTargetTypesTiles, card_dependencies::data_dep_filled::DataDepsFilled};

#[record]
pub struct StatePeerSelectTargets {
    pub selected_index: Vector2Int,
    pub enabled: Option<SelectStates>,
}
impl IState for StatePeerSelectTargets {
    fn id() -> i32 {
        98273477
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Instance
    }
}
impl Hash for StatePeerSelectTargets {
    fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {
        // self.enabled.hash(0);
    }
}
#[derive(Clone, PartialEq, Eq)]
pub enum SelectStates {
    Enabled(AttributeTargetTypesTiles, WorkingState),
    Completed(DataDepsFilled),
}
#[derive(Default, Clone, PartialEq, Eq)]
pub struct WorkingState {
    pub selected_tile: Vector2Int,
    pub selected_entity_ids: Vec<i32>,
    pub selected_card_ids: Vec<i32>,
    pub selected_tile_ids: Vec<i32>,
}
