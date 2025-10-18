use core::collections::vector2_int::Vector2Int;

#[derive(Clone, Copy)]
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
