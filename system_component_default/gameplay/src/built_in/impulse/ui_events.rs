use crate::traits::ui_events::IUIEvent;
use curio_core::{ImpulseCommon, ImpulseScope};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum UIEvents<T>
where
    T: Clone + Sync + IUIEvent + 'static,
{
    Open(T),
    Close(T),
}
impl<T> Display for UIEvents<T>
where
    T: Clone + Sync + IUIEvent + 'static,
{
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
impl<T> ImpulseCommon for UIEvents<T>
where
    T: Clone + Sync + IUIEvent + 'static,
{
    fn id() -> i32
    where
        Self: Sized + 'static,
    {
        1
    }

    fn ownership(&self) -> ImpulseScope
    where
        Self: Sized + 'static,
    {
        ImpulseScope::ConnectedPeers
    }
}
