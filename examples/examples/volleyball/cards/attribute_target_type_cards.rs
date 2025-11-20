use std::fmt::Display;

#[derive(Clone, Copy, Debug)]
pub enum AttributeTargetTypesCards {
    SelectUser,
    SelectOpponent,
    RandomUser,
    RandomOpponent,
    AllUser,
    AllOpponent,
}
impl Display for AttributeTargetTypesCards {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeTargetTypesCards::SelectUser => f.write_str("SelectUser"),
            AttributeTargetTypesCards::SelectOpponent => f.write_str("SelectOpponent"),
            AttributeTargetTypesCards::RandomUser => f.write_str("RandomUser"),
            AttributeTargetTypesCards::RandomOpponent => f.write_str("RandomOpponent"),
            AttributeTargetTypesCards::AllUser => f.write_str("AllUser"),
            AttributeTargetTypesCards::AllOpponent => f.write_str("AllOpponent"),
        }
    }
}
