use crate::cards::{attribute_target_type_cards::AttributeTargetTypesCards, attribute_target_type_entities::AttribtuteTargetTypesEntities, attribute_target_type_players::AtrributeTargetTypesPlayers, attribute_target_type_tiles::AttributeTargetTypesTiles};

/// Data that is empty in order to base filled data off of
#[derive(Clone, Copy)]
pub enum DataDepsEmpty {
    /// A list of players. 0=target_mode
    Players(AtrributeTargetTypesPlayers),
    /// A list of entities. 0=target_mode
    Entities(AttribtuteTargetTypesEntities),
    /// A list of cards. 0=target_mode
    Cards(AttributeTargetTypesCards),
    /// A list of tiles. 0=target_mode
    Tiles(AttributeTargetTypesTiles),
}
