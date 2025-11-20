use crate::collections::game_state::AnyMap;
use egui::{mutex::Mutex, util::id_type_map::TypeId};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    any::{type_name, Any},
    collections::{hash_map::DefaultHasher, HashMap},
    fmt::Display,
    hash::{Hash, Hasher},
};
#[derive(Clone)]
pub struct EventSyncEvent {
    pub target: EventScope,
    id: i32,
    event: Vec<u8>,
}
#[derive(Clone, PartialEq)]
pub enum EventScope {
    All,
    Instance,
    ConnectedHost,
    ConnectedPeers,
}

static mut REGISTERED_GLOBAL_EVENTS: Option<Mutex<HashMap<i32, CreateFN>>> = None;
static mut REGISTERED_GLOBAL_EVENTS_SERIALIZE: Option<Mutex<HashMap<i32, SerializerFn>>> = None;
static mut REGISTERED_GLOBAL_EVENTS_DESERIALIZE: Option<Mutex<HashMap<i32, DeserializerFn>>> = None;

type CreateFN = fn() -> Box<dyn Any>;
type SerializerFn = fn(&dyn Any) -> Vec<u8>;
type DeserializerFn = fn(&[u8]) -> Box<dyn Any>;

pub trait IGameEvent {
    fn get_scope(&self) -> EventScope;
}
pub struct EventQueue {
    cache: AnyMap<i32>,
    sync_events: Vec<EventSyncEvent>,
}
impl EventQueue {
    pub fn register_global_events<T>()
    where
        T: Serialize + DeserializeOwned + 'static,
    {
        println!("AGHHHHJASAH register events! {}", type_name::<T>());
        unsafe {
            if REGISTERED_GLOBAL_EVENTS_SERIALIZE.is_none() {
                REGISTERED_GLOBAL_EVENTS_SERIALIZE = Some(Mutex::new(HashMap::new()));
            }

            let Some(unwrapped) = &REGISTERED_GLOBAL_EVENTS_SERIALIZE else {
                println!("failed to lock REGISTERED_GLOBAL_EVENTS_SERIALIZE");
                return;
            };
            let mut guard = unwrapped.lock();
            let serialize: SerializerFn = |any| {
                let concrete = any.downcast_ref::<T>().unwrap();
                bincode::serialize(concrete).unwrap()
            };
            guard.insert(EventQueue::type_id_to_i32::<T>(), serialize);
        }
        unsafe {
            if REGISTERED_GLOBAL_EVENTS_DESERIALIZE.is_none() {
                REGISTERED_GLOBAL_EVENTS_DESERIALIZE = Some(Mutex::new(HashMap::new()));
            }

            let Some(unwrapped) = &REGISTERED_GLOBAL_EVENTS_DESERIALIZE else {
                println!("failed to lock REGISTERED_GLOBAL_EVENTS_DESERIALIZE");
                return;
            };
            let mut guard = unwrapped.lock();
            let deserialize: DeserializerFn = |bytes| {
                let obj: T = bincode::deserialize(bytes).unwrap();
                Box::new(obj) as Box<dyn Any>
            };
            guard.insert(EventQueue::type_id_to_i32::<T>(), deserialize);
        }
        println!("register STATE_SERIALIZED {}", type_name::<T>());
    }

    pub fn drain_network_sync_events(&mut self) -> Vec<EventSyncEvent> {
        // clone into a new value to return back
        let output = self.sync_events.clone();

        // clear original value
        self.sync_events.clear();

        // return new cloned value
        output
    }
    pub fn apply_network_sync_events(&mut self, sync_events: Vec<EventSyncEvent>) {
        unsafe {
            // get the list of global deserializers
            let Some(unwrapped) = &REGISTERED_GLOBAL_EVENTS_DESERIALIZE else {
                println!("Failed to find REGISTERED_GLOBAL_STATES_DESERIALIZE");
                return;
            };

            // lock the list to avoid other changing it
            let guard = unwrapped.lock();

            // iterate fore ach event in the sync events
            for sync in sync_events {
                // use the the globaly registered values to find a fn to unwrap the bits to the correct type
                let Some(cast_bits_fn) = guard.get(&sync.id) else {
                    println!("Failed to find conversion from bits to type");
                    continue;
                };

                // cast the bits from raw value to the correct type
                let cast_bits = (cast_bits_fn)(&sync.event);

                // get list of events for the id of
                let Some(m) = self.cache.get_mut::<Vec<Box<dyn Any>>, i32>(&sync.id) else {
                    self.cache
                        .insert_any(sync.id, Box::new(vec![Box::new(cast_bits)]));
                    println!("failed to find any for {}", sync.id);
                    continue;
                };

                //
                m.push(cast_bits);
            }
        }
    }

    pub fn new() -> EventQueue {
        EventQueue {
            cache: AnyMap::<i32>::default(),
            sync_events: Vec::new(),
        }
    }

    fn type_id_to_i32<T: 'static>() -> i32 {
        let mut hasher = DefaultHasher::new();
        TypeId::of::<T>().hash(&mut hasher);
        (hasher.finish() & 0xFFFF_FFFF) as i32 // Safe truncation
    }

    pub fn enqueue_event<T: 'static>(&mut self, cast_bits_fn: T)
    where
        T: Clone + IGameEvent + Serialize + DeserializeOwned + Display,
    {
        let scope = cast_bits_fn.get_scope();
        if scope != EventScope::Instance {
            unsafe {
                let Some(unwrapped) = &REGISTERED_GLOBAL_EVENTS_SERIALIZE else {
                    println!("failed to lock REGISTERED_GLOBAL_STATES_SERIALIZE");
                    return;
                };
                let guard = unwrapped.lock();

                let z = guard.get(&EventQueue::type_id_to_i32::<T>()).unwrap();
                let e = EventSyncEvent {
                    target: scope.clone(),
                    id: EventQueue::type_id_to_i32::<T>(),
                    event: ((z)(&cast_bits_fn)),
                };

                self.sync_events.push(e);

                if scope != EventScope::All {
                    println!("bad scope");
                    return;
                }
            }
        }

        let id = EventQueue::type_id_to_i32::<T>();
        if let Some(vec) = self.cache.get_mut::<Vec<Box<dyn Any>>, i32>(&id) {
            // println!("add event append: {}", &cast_bits_fn);
            vec.push(Box::new(cast_bits_fn));
        } else {
            // println!("add event list:  {}", &cast_bits_fn);

            self.cache
                .insert::<Vec<Box<dyn Any>>>(id, vec![Box::new(cast_bits_fn)]);
        }
    }
    pub fn drain_queued_events<T: 'static>(&mut self) -> Vec<T>
    where
        T: 'static + Clone,
    {
        let id = EventQueue::type_id_to_i32::<T>();
        if let Some(x) = self.cache.get_mut::<Vec<Box<dyn Any>>, i32>(&id) {
            let z = EventQueue::extract_vec::<T>(x);
            // let mut vec: Vec<T> = vec![];
            // println!("count {}, other count {}", x.len(), z.len());

            // for t in z {
            //     vec.push(t.clone());

            //     //     let Some(r) = t.downcast_ref::<Box<T>>() else {
            //     //         println!("failed to cast");
            //     //         continue;
            //     //     };
            //     //     let c = *r.clone();
            //     //     vec.push(c);
            //     //     println!("sucess to cast");
            // }
            x.clear();
            z
        } else {
            vec![]
        }
    }
    fn extract_vec<T: 'static + Clone>(items: &Vec<Box<dyn Any>>) -> Vec<T> {
        items
            .iter()
            .filter_map(|item| item.downcast_ref::<T>().cloned())
            .collect()
    }
}
