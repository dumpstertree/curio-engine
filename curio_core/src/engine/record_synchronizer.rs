use crate::{
    static_data::global_states::{get_global_state_deserializer, get_global_state_serializer},
    Curio, RecordCommon, RecordScope,
};
use serde::{Deserialize, Serialize};

/// An representation of a Record that is serialized for rebuild by a Ledger
#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct RecordSynchronizer {
    pub record_id: i32,
    pub record_scope: RecordScope,
    payload: Vec<u8>, // serialized data
}

impl RecordSynchronizer {
    /// Creates an RecordSynchronizer from the provided Record. Will fail if Record is not registered with GlobalRecords
    pub fn serialize<T>(val: &T) -> Option<RecordSynchronizer>
    where
        T: RecordCommon + 'static,
    {
        // pull out any values we need from the IState to record its identity
        let state_id = T::id();
        let state_ownership = T::ownership();

        // convert the state data to raw bytes to send
        let Some(serialized_state) = Self::serialize_from_global(&state_id, val) else {
            return None;
        };

        //
        Some(RecordSynchronizer {
            record_id: state_id,
            payload: serialized_state,
            record_scope: state_ownership,
        })
    }
    /// Creates an Record cast as a Box<RecordCommon> from this RecordSynchronizer. Will fail if Record is not registered with GlobalRecords.
    pub fn deserialize(&self) -> Option<Box<dyn RecordCommon>> {
        // conver the state data into an IState
        let Some(deserialized_state) = Self::deserialize_from_global(&self.record_id, &self.payload) else {
            return None;
        };

        // return the value
        Some(deserialized_state)
    }
}
impl RecordSynchronizer {
    fn deserialize_from_global(id: &i32, bytes: &Vec<u8>) -> Option<Box<dyn RecordCommon>> {
        // get global fn
        let Some(fn_deserialize) = &get_global_state_deserializer(id) else {
            Curio::log(crate::Severity::Warning, "Failed to get GlobalDeserializeFn");
            return None;
        };

        // return result
        Some(fn_deserialize(&bytes.as_slice()))
    }
    fn serialize_from_global<T>(id: &i32, value: &T) -> Option<Vec<u8>>
    where
        T: 'static,
    {
        // get global fn
        let Some(fn_serialize) = &get_global_state_serializer(id) else {
            Curio::log(crate::Severity::Warning, "Failed to get GlobalSerializeFn");
            return None;
        };

        // return result
        Some(fn_serialize(value))
    }
}
