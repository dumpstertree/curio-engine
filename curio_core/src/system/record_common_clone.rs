use crate::RecordCommon;

pub trait RecordCommonClone {
    fn clone_box(&self) -> Box<dyn RecordCommon>;
}
impl<T> RecordCommonClone for T
where
    T: 'static + RecordCommon + Clone,
{
    fn clone_box(&self) -> Box<dyn RecordCommon> {
        Box::new(self.clone())
    }
}
