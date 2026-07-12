use curio_core::Vector2Int;
use std::fmt::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttributeTargetTypesTiles {
    /// Select a tile - Not yet implemented
    SelectAny,
    /// Select a tile on users team
    SelectOnTeamUser,
    /// Select a tile on the opponents team
    SelectOnTeamOpponent,
    /// Random tile on either the user or opponents side
    RandomAny,
    /// Random tile on the users side
    RandomOnTeamUser,
    /// Random tile on the opponents side
    RandomOnTeamOpponent,
    /// Random tile between the values of min and max. Value is explicit but takes into account team rotation
    RandomInRangeGlobal(Vector2Int, Vector2Int),
    /// Random tile between the values of min and max. Value is added to current ball position but takes into account team rotation
    RandomInRangeLocalToBall(Vector2Int, Vector2Int),
    /// Random tile at the value. Value is added to current user position but takes into account team rotation
    RandomInRangeLocalToUser(Vector2Int, Vector2Int),
    /// Select tile between the values of min and max. Value is added to current ball position but takes into account team rotation
    SelectInRangeLocalToBall(Vector2Int, Vector2Int),
    // Select a back line edge on the opponents side
    SelectOpponentBackCorner,
}
impl Display for AttributeTargetTypesTiles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeTargetTypesTiles::SelectAny => f.write_str("SelectAny"),
            AttributeTargetTypesTiles::RandomAny => f.write_str("RandomAny"),
            AttributeTargetTypesTiles::RandomOnTeamUser => f.write_str("RandomOnTeamUser"),
            AttributeTargetTypesTiles::RandomOnTeamOpponent => f.write_str("RandomOnTeamOpponent"),
            AttributeTargetTypesTiles::RandomInRangeGlobal(min, max) => f.write_str(&format!("RandomInRangeGlobal -> min : {}, max: {}", min, max)),
            AttributeTargetTypesTiles::RandomInRangeLocalToBall(min, max) => f.write_str(&format!("RandomInRangeLocal -> min : {}, max: {}", min, max)),
            AttributeTargetTypesTiles::RandomInRangeLocalToUser(min, max) => f.write_str(&format!("RandomInRangeLocal -> min : {}, max: {}", min, max)),
            AttributeTargetTypesTiles::SelectOnTeamUser => f.write_str("SelectOnTeamUser"),
            AttributeTargetTypesTiles::SelectOnTeamOpponent => f.write_str("SelectOnTeamOpponent"),
            AttributeTargetTypesTiles::SelectInRangeLocalToBall(_, _) => f.write_str("SelectInRangeLocal"),
            AttributeTargetTypesTiles::SelectOpponentBackCorner => f.write_str("SelectOpponentBackCorner"),
        }
    }
}
