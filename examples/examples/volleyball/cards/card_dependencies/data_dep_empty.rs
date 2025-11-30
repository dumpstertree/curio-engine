use std::fmt::Display;

use crate::cards::card_attributes_targets::{attribute_target_type_cards::AttributeTargetTypesCards, attribute_target_type_entities::AttribtuteTargetTypesEntities, attribute_target_type_tiles::AttributeTargetTypesTiles};

/// Data that is empty in order to base filled data off of
#[derive(Clone, Copy, Debug)]
pub enum DataDepsEmpty {
    /// A list of entities. 0=target_mode
    Entities(AttribtuteTargetTypesEntities),
    /// A list of cards. 0=target_mode
    Cards(AttributeTargetTypesCards),
    /// A list of tiles. 0=target_mode
    Tiles(AttributeTargetTypesTiles),
}
impl Display for DataDepsEmpty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataDepsEmpty::Entities(tar) => f.write_str(&format!("'Entities' with target type : {}", tar)),
            DataDepsEmpty::Cards(tar) => f.write_str(&format!("'Cards' with target type : {}", tar)),
            DataDepsEmpty::Tiles(tar) => f.write_str(&format!("'Tiles' with target type : {}", tar)),
        }
    }
}
