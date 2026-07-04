use crate::ImpulseCommon;

/// A helper trait for cloning specific for Impulses
pub trait ImpulseClone {
    fn clone_box(&self) -> Box<dyn ImpulseCommon>;
}
impl<T> ImpulseClone for T
where
    T: 'static + ImpulseCommon + Clone,
{
    fn clone_box(&self) -> Box<dyn ImpulseCommon> {
        Box::new(self.clone())
    }
}
