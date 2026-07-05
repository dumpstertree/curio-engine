use crate::RecordCommon;

/// A helper trait for cloning specific for Records
pub trait RecordClone {
    fn clone_box(&self) -> Box<dyn RecordCommon>;
}
impl<T> RecordClone for T
where
    T: 'static + RecordCommon + Clone,
{
    fn clone_box(&self) -> Box<dyn RecordCommon> {
        Box::new(self.clone())
    }
}
