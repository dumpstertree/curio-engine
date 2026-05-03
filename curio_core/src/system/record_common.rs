use crate::{RecordCommonClone, StateOwnerships};
use downcast_rs::{impl_downcast, Downcast};

impl_downcast!(RecordCommon);

pub trait RecordCommon: RecordCommonClone + Downcast {
    fn default_box() -> Box<dyn RecordCommon>
    where
        Self: Sized + Default + 'static,
    {
        Box::new(Self::default())
    }
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
//
