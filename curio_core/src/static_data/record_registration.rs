use crate::RecordCommon;
use std::any::Any;

pub(crate) type ConstructFn = fn() -> Box<dyn RecordCommon>;
pub(crate) type SerializerFn = fn(&dyn Any) -> Vec<u8>;
pub(crate) type DeserializerFn = fn(&[u8]) -> Box<dyn RecordCommon>;

pub struct RecordRegistration {
    pub construct_fn: ConstructFn,
    pub serialize_fn: Option<SerializerFn>,
    pub deserialize_fn: Option<DeserializerFn>,
}
