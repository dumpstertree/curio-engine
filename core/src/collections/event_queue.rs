use crate::{
    collections::game_state::{AnyMap, NetworkSynchEvent},
    dumpster_engine::NetworkModes,
    system::system_game_state::to_bytes,
};
use egui::{mutex::Mutex, util::id_type_map::TypeId};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    any::{type_name, Any},
    collections::{hash_map::DefaultHasher, HashMap},
    fmt::Display,
    hash::{Hash, Hasher},
    vec::Drain,
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
        let x = self.sync_events.clone();
        if x.len() > 0 {
            // println!("drain event count {} ", &x.len());
        }
        self.sync_events.clear();
        x
    }
    pub fn apply_network_sync_events(&mut self, sync_events: Vec<EventSyncEvent>) {
        if sync_events.len() > 0 {
            // println!("apply event count {} ", &sync_events.len());
        }
        unsafe {
            let Some(unwrapped) = &REGISTERED_GLOBAL_EVENTS_DESERIALIZE else {
                println!("failed to lock REGISTERED_GLOBAL_STATES_DESERIALIZE");
                return;
            };
            let guard = unwrapped.lock();

            for sync in sync_events {
                if !guard.contains_key(&sync.id) {
                    println!("nope");
                    return;
                }
                let val = guard.get(&sync.id).unwrap();
                let x = (val)(&sync.event);
                let m = self
                    .cache
                    .get_mut::<Vec<Box<dyn Any>>, i32>(&sync.id)
                    .unwrap();

                m.push(x);
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

    pub fn enqueue_event<T: 'static>(&mut self, val: T)
    where
        T: Clone + IGameEvent + Serialize + DeserializeOwned + Display,
    {
        let scope = val.get_scope();
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
                    event: ((z)(&val)),
                };

                self.sync_events.push(e);

                if scope != EventScope::All {
                    return;
                }
            }
        }

        let id = EventQueue::type_id_to_i32::<T>();
        if let Some(vec) = self.cache.get_mut::<Vec<Box<dyn Any>>, i32>(&id) {
            // println!("add event append: {}", &val);
            vec.push(Box::new(val));
        } else {
            // println!("add event list:  {}", &val);

            self.cache
                .insert::<Vec<Box<dyn Any>>>(id, vec![Box::new(val)]);
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

    // pub fn get_queued_events<T: 'static>(&self) -> &[T]
    // where
    //     T: Clone,
    // {
    //     let id = EventQueue::type_id_to_i32::<T>();
    //     if let Some(x) = self.cache.get::<Vec<Box<dyn Any>>, i32>(&id) {
    //         x.as_slice()
    //     } else {
    //         &[]
    //     }
    // }
    // pub fn clear_queued_events<T: 'static>(&mut self) {
    //     let id = EventQueue::type_id_to_i32::<T>();
    //     if let Some(x) = self.cache.get_mut::<Vec<Box<dyn ANy>>, i32>(&id) {
    //         x.clear();
    //     }
    // }
}
