use crate::{
    engine::impulse_common::ImpulseCommon,
    static_data::global_events::{get_global_event_deserializer, get_global_event_serializer},
    Curio, ImpulseScope,
};
use std::any::Any;
#[derive(Clone)]

/// An representation of an Impulse that is serialized for rebuild by a Nerve
pub struct ImpulseSynchronizer {
    pub impulse_id: i32,
    pub impulse_scope: ImpulseScope,
    payload: Vec<u8>,
}
impl ImpulseSynchronizer {
    /// Creates an ImpulseSynchronizer from the provided Impulse. Will fail if Impulse is not registered with GlobalImpulses
    pub fn serialize<T>(val: &T) -> Option<ImpulseSynchronizer>
    where
        T: ImpulseCommon + 'static,
    {
        // pull out any values we need from the IState to record its identity
        let event_id = T::id();
        let event_ownership = val.ownership();

        // convert the state data to raw bytes to send
        let Some(serialized_state) = Self::serialize_from_global(&event_id, val) else {
            return None;
        };

        //
        Some(ImpulseSynchronizer {
            impulse_id: event_id,
            payload: serialized_state,
            impulse_scope: event_ownership,
        })
    }

    /// Creates an Impulse cast as a Box<Any> from this ImpulseSynchronizer. Will fail if Impulse is not registered with GlobalImpulses.
    pub fn deserialize(&self) -> Option<Box<dyn Any>> {
        Self::deserialize_from_global(&self.impulse_id, &self.payload)
    }
}
impl ImpulseSynchronizer {
    fn deserialize_from_global(id: &i32, bytes: &Vec<u8>) -> Option<Box<dyn Any>> {
        // get global fn
        let Some(fn_deserialize) = &get_global_event_deserializer(id) else {
            Curio::log(crate::Severity::Warning, "Failed to get GlobalDeserializeFn");
            return None;
        };

        // return result
        Some(fn_deserialize(&bytes.as_slice()))
    }

    fn serialize_from_global<T>(id: &i32, value: &T) -> Option<Vec<u8>>
    where
        T: ImpulseCommon + 'static,
    {
        // get global fn
        let Some(fn_serialize) = &get_global_event_serializer(id) else {
            Curio::log(crate::Severity::Warning, "Failed to get GlobalSerializeFn");
            return None;
        };

        // return result
        Some(fn_serialize(value))
    }
}
