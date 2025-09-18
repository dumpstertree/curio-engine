use crate::cards::{attribute_target_type_cards::AttributeTargetTypesCards, attribute_target_type_entities::AttribtuteTargetTypesEntities, attribute_target_type_players::AtrributeTargetTypesPlayers, attribute_target_type_tiles::AttributeTargetTypesTiles, data_dep_empty::DataDepsEmpty};

#[derive(Clone)]
pub enum CardAttributeEvents {
    /// Add cards to hand, 0=count, 1=players
    DrawCards(i32, AtrributeTargetTypesPlayers),
    /// Add cards from hand, 0=count, 1=cards
    DiscardCards(AttributeTargetTypesCards),
    /// Move entities to location, 0=entities, 1=location
    MoveEntity(AttribtuteTargetTypesEntities, AttributeTargetTypesTiles),
    /// Move ball, 0=distance
    MoveBallForward(i32),
    /// Move ball, 0=energy, 1=entities
    GainEnergy(i32, AttribtuteTargetTypesEntities),
    // Refill energy to max, 0=entities
    RefillEnergy(AttribtuteTargetTypesEntities),
}
impl CardAttributeEvents {
    /// get the required dependencies that need to be passed in
    pub fn get_data_dependencies_empty(&self) -> Vec<DataDepsEmpty> {
        match self {
            CardAttributeEvents::DrawCards(_, t0) => vec![DataDepsEmpty::Players(*t0)],
            CardAttributeEvents::DiscardCards(t0) => vec![DataDepsEmpty::Cards(*t0)],
            CardAttributeEvents::MoveEntity(t0, t1) => vec![DataDepsEmpty::Entities(*t0), DataDepsEmpty::Tiles(*t1)],
            CardAttributeEvents::MoveBallForward(_) => vec![],
            CardAttributeEvents::GainEnergy(_, t0) => vec![DataDepsEmpty::Entities(*t0)],
            CardAttributeEvents::RefillEnergy(t0) => vec![DataDepsEmpty::Entities(*t0)],
        }
    }
}
