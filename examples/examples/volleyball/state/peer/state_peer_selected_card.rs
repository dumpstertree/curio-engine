use curio_core::StateOwnerships;
use record_serializable::record_serializable;
#[derive(Hash, PartialEq, Eq)]
#[record_serializable(serializable,ownership = StateOwnerships::Instance)]
pub struct StatePeerSelectedCards {
    pub index: i32,
}
