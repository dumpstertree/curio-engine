use curio_core::StateOwnerships;
use record_serializable::record_serializable;
use std::hash::Hash;

#[derive(Hash, PartialEq, Eq)]
#[record_serializable(serializable,ownership = StateOwnerships::Host)]
pub struct StateCurrency {
    pub currency: i32,
}
