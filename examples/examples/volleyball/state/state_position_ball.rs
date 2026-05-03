use curio_core::StateOwnerships;
use record_serializable::record_serializable;
#[derive(Hash, PartialEq, Eq)]
#[record_serializable(serializable,ownership = StateOwnerships::Host)]
pub struct StatePositionBall {
    pub row: i32,
    pub column: i32,
}
