use std::fmt::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttribtuteTargetTypesEntities {
    User,
    Select,
    RandomAny,
    RandomOpponent,
}

impl Display for AttribtuteTargetTypesEntities {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttribtuteTargetTypesEntities::User => f.write_str("User"),
            AttribtuteTargetTypesEntities::Select => f.write_str("Select"),
            AttribtuteTargetTypesEntities::RandomAny => f.write_str("RandomAny"),
            AttribtuteTargetTypesEntities::RandomOpponent => f.write_str("RandomOpponent"),
        }
    }
}
