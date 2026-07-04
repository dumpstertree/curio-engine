use crate::{AsAny, ImpulseClone, ImpulseScope};

/// Trait required to use an object as an Impulse
pub trait ImpulseCommon: AsAny + ImpulseClone + Sync {
    fn default_box(self) -> Box<dyn ImpulseCommon>
    where
        Self: Sized + Default + 'static,
    {
        Box::new(self)
    }

    /// The id for a value of an Impulse. For an enum this should be the enums backing value.
    fn id() -> i32
    where
        Self: Sized + 'static;

    /// The ownership for a value of an Impulse. This will direct where this Impulse goes when invoked
    fn ownership(&self) -> ImpulseScope
    where
        Self: Sized + 'static;
}

impl<T: ImpulseCommon + 'static> AsAny for T {
    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
    fn as_mut_any(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }
}
