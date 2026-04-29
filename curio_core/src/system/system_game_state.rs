use crate::collections::state_ownerships::StateOwnerships;
use std::any::TypeId;
use std::hash::{DefaultHasher, Hash, Hasher};

pub trait RecordCommon: IStateClone + IStateHash + DowncastSync {
    fn default_box() -> Box<dyn RecordCommon>
    where
        Self: Sized + Default + 'static,
    {
        Box::new(Self::default())
    }

    // fn id() -> i32
    // where
    //     Self: Sized + 'static,
    // {
    //     let mut hasher = DefaultHasher::new();
    //     TypeId::of::<Self>().hash(&mut hasher);
    //     hasher.finish() as i32
    // }

    fn id() -> i32
    where
        Self: Sized + 'static;
    fn ownership() -> StateOwnerships
    where
        Self: Sized + 'static,
    {
        StateOwnerships::Instance
    }
}

// clone helper for trait objects
pub trait IStateClone {
    fn clone_box(&self) -> Box<dyn RecordCommon>;
}
impl<T> IStateClone for T
where
    T: 'static + RecordCommon + Clone,
{
    fn clone_box(&self) -> Box<dyn RecordCommon> {
        Box::new(self.clone())
    }
}

// -----------------------------
// Object-safe hash -> returns u64
// -----------------------------
pub trait IStateHash {
    /// Return a stable u64 fingerprint for this concrete state value.
    /// Implemented by default via a DefaultHasher for types that impl `Hash`.
    fn hash_dyn_u64(&self) -> u64;
}

impl<T> IStateHash for T
where
    T: 'static + RecordCommon + Hash,
{
    fn hash_dyn_u64(&self) -> u64 {
        let mut h = DefaultHasher::new();
        // Use the concrete Hash impl of the type
        Hash::hash(self, &mut h);
        h.finish()
    }
}

use downcast_rs::{impl_downcast, DowncastSync};
impl_downcast!(sync RecordCommon);
