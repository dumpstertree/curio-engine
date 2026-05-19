use crate::{AsAny, EventScope, IEventClone};
use std::fmt::Display;

pub trait IGameEvent: Display + AsAny + IEventClone + Sync {
    fn default_box(self) -> Box<dyn IGameEvent>
    where
        Self: Sized + Default + 'static,
    {
        Box::new(self)
    }

    fn id() -> i32
    where
        Self: Sized + 'static;
    fn ownership(&self) -> EventScope
    where
        Self: Sized + 'static;
}

impl<T: IGameEvent + 'static> AsAny for T {
    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
    fn as_mut_any(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }
}
