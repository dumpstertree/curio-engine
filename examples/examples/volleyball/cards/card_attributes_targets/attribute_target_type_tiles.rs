use core::collections::vector2_int::Vector2Int;
use std::fmt::Display;

#[derive(Clone, Copy, Debug)]
pub enum AttributeTargetTypesTiles {
    /// Select a tile - Not yet implemented
    Select,
    /// Random tile on either the user or opponents side
    RandomAny,
    /// Random tile on the users side
    RandomOnTeamUser,
    /// Random tile on the opponents side
    RandomOnTeamOpponent,
    /// Random tile between the values of min and max. Value is explicit but takes into account team rotation
    RandomInRangeGlobal(Vector2Int, Vector2Int),
    /// Random tile between the values of min and max. Value is added to current ball position but takes into account team rotation
    RandomInRangeLocal(Vector2Int, Vector2Int),
}
impl Display for AttributeTargetTypesTiles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeTargetTypesTiles::Select => f.write_str("Select"),
            AttributeTargetTypesTiles::RandomAny => f.write_str("RandomAny"),
            AttributeTargetTypesTiles::RandomOnTeamUser => f.write_str("RandomOnTeamUser"),
            AttributeTargetTypesTiles::RandomOnTeamOpponent => f.write_str("RandomOnTeamOpponent"),
            AttributeTargetTypesTiles::RandomInRangeGlobal(min, max) => f.write_str(&format!("RandomInRangeGlobal -> min : {}, max: {}", min, max)),
            AttributeTargetTypesTiles::RandomInRangeLocal(min, max) => f.write_str(&format!("RandomInRangeLocal -> min : {}, max: {}", min, max)),
        }
    }
}
