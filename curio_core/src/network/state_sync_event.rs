use serde::{Deserialize, Serialize};

use crate::{
    static_data::global_states::{get_global_state_deserializer, get_global_state_serializer},
    RecordCommon, StateOwnerships,
};

// The "erased" event you actually store in Vec
#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct StateSyncEvent {
    pub id: i32,
    pub payload: Vec<u8>, // serialized data
    pub ownership: StateOwnerships,
}

impl StateSyncEvent {
    pub fn serialize<T>(val: &T) -> Option<StateSyncEvent>
    where
        T: RecordCommon + 'static,
    {
        // pull out any values we need from the IState to record its identity
        let state_id = T::id();
        let state_ownership = T::ownership();

        // convert the state data to raw bytes to send
        let Some(serialized_state) = Self::serialize_sync_event(&state_id, val) else {
            return None;
        };

        //
        Some(StateSyncEvent {
            id: state_id,
            payload: serialized_state,
            ownership: state_ownership,
        })
    }
    pub fn deserialize(&self) -> Option<Box<dyn RecordCommon>> {
        // conver the state data into an IState
        let Some(deserialized_state) = Self::deserialize_sync_event(&self.id, &self.payload) else {
            return None;
        };

        // return the value
        Some(deserialized_state)
    }
}
impl StateSyncEvent {
    fn deserialize_sync_event(id: &i32, bytes: &Vec<u8>) -> Option<Box<dyn RecordCommon>> {
        // get global fn
        let Some(fn_deserialize) = &get_global_state_deserializer(id) else {
            panic!("Failed to get GlobalDeserializeFn");
        };

        // return result
        Some(fn_deserialize(&bytes.as_slice()))
    }
    fn serialize_sync_event<T>(id: &i32, value: &T) -> Option<Vec<u8>>
    where
        T: 'static,
    {
        // get global fn
        let Some(fn_serialize) = &get_global_state_serializer(id) else {
            panic!("Failed to get GlobalSerializeFn");
        };

        // return result
        Some(fn_serialize(value))
    }
}
