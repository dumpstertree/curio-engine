use crate::cards::{card_attributes_targets::attribute_target_type_entities::AttribtuteTargetTypesEntities, card_dependencies::data_dep_empty::DataDepsEmpty, enums::attribute_clear_flag::ModifierClearFlag};

#[derive(Clone)]
pub enum CardAttributeModifiers {
    /// Edit available energy for an Entity or Group. 0=clear, 1=entity_ids, 2=count
    EditEnergyForEntities(ModifierClearFlag, AttribtuteTargetTypesEntities, i32),
    /// Edit available energy for an Entity or Group. 0=clear, 1=entity_ids, 2=count
    EditRangeForEntities(ModifierClearFlag, AttribtuteTargetTypesEntities, i32),
    /// Edit available energy for an Entity or Group. 0=clear, 1=entity_ids, 2=count
    EditCostForEntities(ModifierClearFlag, AttribtuteTargetTypesEntities, i32),
}
impl CardAttributeModifiers {
    /// get the data dependencies that need to be passed in
    pub fn get_data_dependencies_empty(&self) -> Vec<DataDepsEmpty> {
        match self {
            CardAttributeModifiers::EditEnergyForEntities(_, t0, _) => vec![DataDepsEmpty::Entities(*t0)],
            CardAttributeModifiers::EditRangeForEntities(_, t0, _) => vec![DataDepsEmpty::Entities(*t0)],
            CardAttributeModifiers::EditCostForEntities(_, t0, _) => vec![DataDepsEmpty::Entities(*t0)],
        }
    }
}
