use crate::{RecordClone, RecordOverride, RecordScope};
use downcast_rs::{impl_downcast, Downcast};

//uhh... tbd
impl_downcast!(RecordCommon);

/// Trait required to use an object as an Record
pub trait RecordCommon: RecordClone + Downcast + RecordOverride {
    /// Pretty name for the Record. Does not need to be unique
    fn name(&self) -> String;

    /// The id for a value of an Record
    fn id() -> i32
    where
        Self: Sized + 'static;

    /// The ownership for a Record. Dictates where data is passed to the CurioNetwork
    fn ownership() -> RecordScope
    where
        Self: Sized + 'static,
    {
        RecordScope::Instance
    }

    fn default_box() -> Box<dyn RecordCommon>
    where
        Self: Sized + Default + 'static,
    {
        Box::new(Self::default())
    }
}
