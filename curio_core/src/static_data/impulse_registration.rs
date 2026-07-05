use crate::ImpulseCommon;
use std::any::Any;

pub(crate) type CustructorFn = fn() -> Box<dyn ImpulseCommon>;
pub(crate) type SerializerFn = fn(&dyn Any) -> Vec<u8>;
pub(crate) type DeserializerFn = fn(&[u8]) -> Box<dyn Any>;

pub struct ImpulseRegistration {
    pub construct_fn: CustructorFn,
    pub serialize_fn: SerializerFn,
    pub deserialize_fn: DeserializerFn,
}
