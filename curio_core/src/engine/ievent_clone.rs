use crate::IGameEvent;

// clone helper for trait objects
pub trait IEventClone {
    fn clone_box(&self) -> Box<dyn IGameEvent>;
}
impl<T> IEventClone for T
where
    T: 'static + IGameEvent + Clone,
{
    fn clone_box(&self) -> Box<dyn IGameEvent> {
        Box::new(self.clone())
    }
}
