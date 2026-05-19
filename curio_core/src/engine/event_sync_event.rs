use std::any::Any;

use crate::{
    engine::igame_event::IGameEvent,
    static_data::global_events::{get_global_event_deserializer, get_global_event_serializer},
    EventScope,
};
#[derive(Clone)]
pub struct EventSyncEvent {
    pub id: i32,
    pub payload: Vec<u8>,
    pub ownership: EventScope,
}
impl EventSyncEvent {
    pub fn serialize<T>(val: &T) -> Option<EventSyncEvent>
    where
        T: IGameEvent + 'static,
    {
        // pull out any values we need from the IState to record its identity
        let event_id = T::id();
        let event_ownership = val.ownership();

        // convert the state data to raw bytes to send
        let Some(serialized_state) = Self::serialize_sync_event(&event_id, val) else {
            return None;
        };

        //
        Some(EventSyncEvent {
            id: event_id,
            payload: serialized_state,
            ownership: event_ownership,
        })
    }
    pub fn deserialize(&self) -> Option<Box<dyn Any>> {
        // conver the state data into an IState
        let Some(deserialized_state) = Self::deserialize_sync_event(&self.id, &self.payload) else {
            return None;
        };

        // return the value
        Some(deserialized_state)
    }
}
impl EventSyncEvent {
    fn deserialize_sync_event(id: &i32, bytes: &Vec<u8>) -> Option<Box<dyn Any>> {
        // get global fn
        let Some(fn_deserialize) = &get_global_event_deserializer(id) else {
            println!("Failed to get GlobalDeserializeFn");
            return None;
        };

        // return result
        Some(fn_deserialize(&bytes.as_slice()))
    }
    fn serialize_sync_event<T>(id: &i32, value: &T) -> Option<Vec<u8>>
    where
        T: IGameEvent + 'static,
    {
        // get global fn
        let Some(fn_serialize) = &get_global_event_serializer(id) else {
            println!("Failed to get GlobalSerializeFn");
            return None;
        };

        // return result
        Some(fn_serialize(value))
    }
}
