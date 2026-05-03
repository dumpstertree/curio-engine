use curio_core::{StateOwnerships, Vector2Int};
use record_serializable::record_serializable;
use std::hash::Hash;

use crate::cards::{card_attributes_targets::attribute_target_type_tiles::AttributeTargetTypesTiles, card_dependencies::data_dep_filled::DataDepsFilled};

#[record_serializable(ownership = StateOwnerships::Instance)]
pub struct StatePeerSelectTargets {
    pub selected_index: Vector2Int,
    pub enabled: Option<SelectStates>,
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
