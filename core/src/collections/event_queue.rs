use egui::{mutex::Mutex, util::id_type_map::TypeId};
use rapier3d::parry::simba::scalar::SupersetOf;
use rusty_spine::c::Str;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    any::{self, type_name, Any},
    collections::{hash_map::DefaultHasher, HashMap},
    fmt::Display,
    hash::{Hash, Hasher},
};

use crate::{
    collections::{any_map::AnyMap, network_capabilities},
    dumpster_engine::NetworkModes,
    static_data::global_events::{get_global_event_deserializer, get_global_event_serializer},
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

#[derive(Clone, PartialEq)]
pub enum EventScope {
    All,
    Instance,
    ConnectedHost,
    ConnectedPeers,
}

pub trait IGameEvent: AsAny + IEventClone + /*IEventHash +*/ Sync {
    fn default_box(self) -> Box<dyn IGameEvent>
    where
        Self: Sized + Default + 'static,
    {
        Box::new(self)
    }

    fn id() -> i32
    where
        Self: Sized + 'static;
    fn ownership(&self) -> EventScope
    where
        Self: Sized + 'static;
}

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

// -----------------------------
// Object-safe hash -> returns u64
// -----------------------------
// pub trait IEventHash {
//     /// Return a stable u64 fingerprint for this concrete state value.
//     /// Implemented by default via a DefaultHasher for types that impl `Hash`.
//     fn hash_dyn_u64(&self) -> u64;
// }

// impl<T> IEventHash for T
// where
//     // T: 'static + IGameEvent + Hash,
//     T: 'static + IGameEvent,
// {
//     fn hash_dyn_u64(&self) -> u64 {
//         let mut h = DefaultHasher::new();
//         // Use the concrete Hash impl of the type
//         Hash::hash(&self.id(), &mut h);
//         h.finish()
//     }
// }

#[derive(Clone)]
pub struct NetworkCapabilities {
    /// level of privilege this instance has
    pub privilege: NetworkModes,

    /// events waiting to be drained and sent to other game states
    pub ouput_sync_events: Vec<EventSyncEvent>,
}
// Public - Fns
impl NetworkCapabilities {
    pub fn has_write_privilege(&self, state_ownership: EventScope) -> bool {
        // if the state is owned by the instance we never need to send it
        match state_ownership {
            EventScope::Instance => return false,
            _ => {}
        }
        // // if the state is owned by the host we right it if we have host privilege
        // match self.privilege {
        //     NetworkModes::LocalHost => return true,
        //     NetworkModes::OnlineHost => return true,
        //     NetworkModes::LocalPeer => return false,
        //     NetworkModes::OnlinePeer => return false,
        // }
        true
    }
    pub fn drain_sync_events(&mut self) -> Vec<EventSyncEvent> {
        let result = self.ouput_sync_events.clone();
        self.ouput_sync_events.clear();
        result
    }
    pub fn enqueue_sync_events(&mut self, event: EventSyncEvent) {
        self.ouput_sync_events.push(event);
    }
}
// Static - Fns
impl NetworkCapabilities {}
impl NetworkCapabilities {
    pub fn new(privilige: NetworkModes) -> NetworkCapabilities {
        NetworkCapabilities { privilege: privilige, ouput_sync_events: Vec::new() }
    }
}

pub struct EventQueue {
    pub name: String,
    cache: HashMap<i32, Queue>,
    network_capabilities: Option<NetworkCapabilities>,
}

impl EventQueue {
    pub fn new(name: &str, privilege: NetworkModes) -> Self {
        EventQueue {
            name: String::from(name),
            cache: HashMap::new(),
            network_capabilities: Some(NetworkCapabilities::new(privilege)),
        }
    }

    pub fn enqueue_event<T>(&mut self, event: T)
    where
        T: Clone + IGameEvent + Serialize + DeserializeOwned + Display + 'static,
    {
        println!("ENQUEUE {}", event);

        // Insert or push into Queue
        self.cache
            .entry(T::id())
            .or_insert_with(Queue::new)
            .push(event.clone());

        // Network Sync Logic
        let Some(network_capabilities) = &mut self.network_capabilities else {
            println!("no network");
            return;
        };

        if !network_capabilities.has_write_privilege(event.ownership()) {
            return;
        }

        if let Some(ser) = EventSyncEvent::serialize::<T>(&event) {
            network_capabilities.enqueue_sync_events(ser);
        }
    }

    pub fn drain_queued_events<T: 'static>(&mut self) -> Vec<T>
    where
        T: IGameEvent + Display + Clone + 'static,
    {
        let key = T::id();
        match self.cache.get_mut(&key) {
            Some(queue) => {
                let drained = queue.drain::<T>();
                drained
            }
            None => {
                println!("no queue found for id {}", key);
                Vec::new()
            }
        }
    }

    pub fn try_apply_network_sync_events(&mut self, sync_events: Vec<EventSyncEvent>) {
        for sync in sync_events {
            let Some(payload) = sync.deserialize() else {
                println!("failed deserialize");
                continue;
            };

            self.cache
                .entry(sync.id)
                .or_insert_with(Queue::new)
                .push_boxed(payload);
        }
    }

    pub fn try_drain_network_sync_events(&mut self) -> Vec<EventSyncEvent> {
        let Some(network_capabilities) = &mut self.network_capabilities else {
            return Vec::new();
        };

        network_capabilities.drain_sync_events()
    }
}

pub trait AsAny {
    fn as_any(&self) -> Option<&dyn std::any::Any> {
        None
    }
    fn as_mut_any(&mut self) -> Option<&mut dyn std::any::Any> {
        None
    }
}

impl<T: IGameEvent + 'static> AsAny for T {
    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
    fn as_mut_any(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }
}

pub struct Queue {
    items: Vec<Box<dyn Any>>,
}

impl Queue {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Push a typed event into the queue
    pub fn push<T: 'static>(&mut self, event: T) {
        self.items.push(Box::new(event));
    }
    pub fn push_boxed(&mut self, event: Box<dyn Any>) {
        self.items.push(event);
    }

    /// Drain all events of type T, removing them from the queue,
    /// returning owned T values.
    pub fn drain<T: 'static>(&mut self) -> Vec<T> {
        let mut out = Vec::new();
        let mut remaining = Vec::new();

        for boxed in self.items.drain(..) {
            // Try to downcast and keep only matching items
            match boxed.downcast::<T>() {
                Ok(boxed_t) => out.push(*boxed_t),
                Err(other) => remaining.push(other),
            }
        }

        self.items = remaining;
        out
    }

    /// Get a read-only view of all events of type T (without draining)
    pub fn get_all<T: 'static>(&self) -> Vec<&T> {
        self.items
            .iter()
            .filter_map(|item| item.downcast_ref::<T>())
            .collect()
    }

    /// Returns whether any item of type T is in the queue
    pub fn has_type<T: 'static>(&self) -> bool {
        self.items.iter().any(|item| item.is::<T>())
    }

    /// Return total items in queue (all types)
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
