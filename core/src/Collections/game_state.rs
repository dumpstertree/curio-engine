use std::any::type_name;
use std::hash::{Hash, Hasher};
use std::vec;
use std::{any::Any, borrow::Borrow, collections::HashMap};

use egui::mutex::Mutex;
use serde::de::DeserializeOwned;

use crate::dumpster_engine::NetworkModes;
use crate::system::system_game_state::IState;

pub struct NetworkSynchEvent {
    id: i32,
    payload: Vec<u8>,
}

use serde::{Deserialize, Serialize};

// The "erased" event you actually store in Vec
#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct StateSyncEvent {
    pub id: i32,
    pub payload: Vec<u8>, // serialized data
}

trait GameStateCommon {
    fn get<T>()
    where
        T: Default;
    fn edit<T>()
    where
        T: Default;
}
pub enum StateOwnerships {
    Instance,
    Host,
}

static mut REGISTERED_GLOBAL_STATES: Option<Mutex<HashMap<i32, CreateFN>>> = None;
static mut REGISTERED_GLOBAL_STATES_SERIALIZE: Option<Mutex<HashMap<i32, SerializerFn>>> = None;
static mut REGISTERED_GLOBAL_STATES_DESERIALIZE: Option<Mutex<HashMap<i32, DeserializerFn>>> = None;

type CreateFN = fn() -> Box<dyn IState>;
type SerializerFn = fn(&dyn Any) -> Vec<u8>;
type DeserializerFn = fn(&[u8]) -> Box<dyn IState>;

#[derive(Clone)]
pub struct GameState {
    has_network_capabilities: bool,
    pub instance_id: i32,
    pub all_instance_id: Vec<i32>,
    edited_state: Vec<StateSyncEvent>,
    fn_deserialize: HashMap<i32, DeserializerFn>,
    fn_serialize: HashMap<i32, SerializerFn>,
    pub(crate) cache: StateMap<i32>,
    pub network_mode: NetworkModes,
}

impl GameState {
    pub fn register_global_states<T>()
    where
        T: Any + IState + Default,
    {
        unsafe {
            if REGISTERED_GLOBAL_STATES.is_none() {
                REGISTERED_GLOBAL_STATES = Some(Mutex::new(HashMap::new()));
            }

            let Some(unwrapped) = &REGISTERED_GLOBAL_STATES else {
                println!("failed to lock REGISTERED_GLOBAL_STATES");
                return;
            };
            let mut guard = unwrapped.lock();
            let create: fn() -> Box<dyn IState> = || return T::default_box();
            guard.insert(T::id(), create);
        }
        println!("register STATE {}", type_name::<T>());
    }
    pub fn register_global_states_serializable<T>()
    where
        T: IState + Serialize + DeserializeOwned + Default + 'static,
    {
        unsafe {
            if REGISTERED_GLOBAL_STATES.is_none() {
                REGISTERED_GLOBAL_STATES = Some(Mutex::new(HashMap::new()));
            }

            let Some(unwrapped) = &REGISTERED_GLOBAL_STATES else {
                println!("failed to lock REGISTERED_GLOBAL_STATES");
                return;
            };
            let mut guard = unwrapped.lock();
            let create: fn() -> Box<dyn IState> = || return Box::new(T::default());
            guard.insert(T::id(), create);
        }
        unsafe {
            if REGISTERED_GLOBAL_STATES_SERIALIZE.is_none() {
                REGISTERED_GLOBAL_STATES_SERIALIZE = Some(Mutex::new(HashMap::new()));
            }

            let Some(unwrapped) = &REGISTERED_GLOBAL_STATES_SERIALIZE else {
                println!("failed to lock REGISTERED_GLOBAL_STATES_SERIALIZE");
                return;
            };
            let mut guard = unwrapped.lock();
            let serialize: SerializerFn = |any| {
                let concrete = any.downcast_ref::<T>().unwrap();
                bincode::serialize(concrete).unwrap()
            };
            guard.insert(T::id(), serialize);
        }
        unsafe {
            if REGISTERED_GLOBAL_STATES_DESERIALIZE.is_none() {
                REGISTERED_GLOBAL_STATES_DESERIALIZE = Some(Mutex::new(HashMap::new()));
            }

            let Some(unwrapped) = &REGISTERED_GLOBAL_STATES_DESERIALIZE else {
                println!("failed to lock REGISTERED_GLOBAL_STATES_DESERIALIZE");
                return;
            };
            let mut guard = unwrapped.lock();
            let deserialize: DeserializerFn = |bytes| {
                let obj: T = bincode::deserialize(bytes).unwrap();
                Box::new(obj) as Box<dyn IState>
            };
            guard.insert(T::id(), deserialize);
        }
        println!("register STATE_SERIALIZED {}", type_name::<T>());
    }

    pub fn apply_network_sync_events(&mut self, sync: Vec<StateSyncEvent>) {
        // println!("apply state sync ");
        for evnt in sync {
            if !self.fn_deserialize.contains_key(&evnt.id) {
                println!("Unknown id");
                continue;
            }

            let result = self.fn_deserialize[&evnt.id](evnt.payload.as_slice());
            self.cache.insert_any(evnt.id, result);
        }
    }
    pub fn drain_network_sync_events(&mut self) -> Vec<StateSyncEvent> {
        let x = self.edited_state.clone();
        self.edited_state.clear();
        return x;
    }
    pub fn change_network_mode(&mut self, mode: NetworkModes) {
        self.network_mode = mode;
    }
    pub fn new_single_instance(states: Vec<(i32, Box<dyn IState>)>) -> GameState {
        let mut fn_serialize = HashMap::<i32, SerializerFn>::default();
        let mut fn_deserialize = HashMap::<i32, DeserializerFn>::default();

        let mut cache = StateMap::new();
        for state in states {
            cache.insert_any(state.0, state.1);
        }

        GameState {
            has_network_capabilities: false,
            instance_id: -1,
            all_instance_id: vec![],
            edited_state: Vec::new(),
            cache: cache,
            network_mode: NetworkModes::LocalHost,
            fn_deserialize: fn_deserialize,
            fn_serialize: fn_serialize,
        }
    }
    pub fn new(network_mode: NetworkModes, instance_id: i32, all_instance_id: Vec<i32>) -> GameState {
        let mut cache = StateMap::<i32>::default();
        let mut fn_serialize = HashMap::<i32, SerializerFn>::default();
        let mut fn_deserialize = HashMap::<i32, DeserializerFn>::default();

        unsafe {
            let Some(unwrapped) = &REGISTERED_GLOBAL_STATES else {
                panic!("Failed to unwrap state");
            };
            let guard = unwrapped;
            let z = guard.lock();

            for x in z.iter() {
                cache.insert_any(x.0.clone(), x.1());
            }

            let Some(unwrapped) = &REGISTERED_GLOBAL_STATES_SERIALIZE else {
                panic!("Failed to unwrap state");
            };
            let guard = unwrapped;
            let z = guard.lock();

            for x in z.iter() {
                fn_serialize.insert(x.0.clone(), *x.1);
            }

            let Some(unwrapped) = &REGISTERED_GLOBAL_STATES_DESERIALIZE else {
                panic!("Failed to unwrap state");
            };
            let guard = unwrapped;
            let z = guard.lock();

            for x in z.iter() {
                fn_deserialize.insert(x.0.clone(), *x.1);
            }
        }
        GameState {
            has_network_capabilities: true,
            instance_id: instance_id,
            all_instance_id: all_instance_id,
            edited_state: Vec::new(),
            cache: cache,
            network_mode: network_mode,
            fn_deserialize: fn_deserialize,
            fn_serialize: fn_serialize,
        }
    }

    fn has_write_permision(mode: &NetworkModes, ownership: &StateOwnerships) -> bool {
        match mode {
            NetworkModes::LocalHost => true,
            NetworkModes::LocalPeer => match ownership {
                StateOwnerships::Instance => true,
                StateOwnerships::Host => false,
            },
            NetworkModes::OnlineHost => true,
            NetworkModes::OnlinePeer => match ownership {
                StateOwnerships::Instance => true,
                StateOwnerships::Host => false,
            },
        }
    }
    fn has_push_permision(mode: &NetworkModes, ownership: &StateOwnerships) -> bool {
        match ownership {
            StateOwnerships::Instance => false,
            StateOwnerships::Host => match mode {
                NetworkModes::LocalHost => true,
                NetworkModes::LocalPeer => false,
                NetworkModes::OnlineHost => true,
                NetworkModes::OnlinePeer => false,
            },
        }
    }

    pub fn edit<T: 'static>(&mut self, edit: impl Fn(&mut T))
    where
        T: IState,
        T: Clone,
    {
        let id = T::id();
        let Some(mut val) = self.cache.get_mut::<T, i32>(&id) else {
            // return;
            panic!("Requested unknown value of type {}", type_name::<T>());
        };

        if self.has_network_capabilities && !GameState::has_write_permision(&self.network_mode, &T::ownership()) {
            println!("did not have write permissions for {}", type_name::<T>());
            return;
        }

        edit(&mut val);

        if self.has_network_capabilities && GameState::has_push_permision(&self.network_mode, &T::ownership()) {
            let id2 = T::id();
            let data = self.fn_serialize[&id2](&self.get_value2::<T>());
            self.edited_state
                .push(StateSyncEvent { id: id2.clone(), payload: data });
        }
    }

    pub fn get_value2<T: 'static>(&self) -> T
    where
        T: IState,
        T: Clone,
    {
        let id = T::id();
        let Some(val) = self.cache.get::<T, i32>(&id) else {
            panic!("Requested unknown value of type {}", type_name::<T>());
            // return T::default();
        };

        return val.clone();
    }
}

#[derive(Default)]
pub struct AnyMap<K>(HashMap<K, Box<dyn Any>>);

#[derive(Debug)]
pub enum GetError {
    EmptyKey,
    MismatchedType,
}

impl<K: Hash + Eq> AnyMap<K> {
    pub fn insert<T: Any>(&mut self, key: K, value: T) {
        self.0.insert(key, Box::new(value));
    }
    pub fn insert_any(&mut self, key: K, value: Box<dyn Any>) {
        self.0.insert(key, value);
    }

    pub fn get<T: Any, Q: ?Sized + Hash + Eq>(&self, key: &Q) -> Option<&T>
    where
        K: Borrow<Q>,
    {
        self.0.get(key)?.downcast_ref()
    }
    pub fn get_mut<T: Any, Q: ?Sized + Hash + Eq>(&mut self, key: &Q) -> Option<&mut T>
    where
        K: Borrow<Q>,
    {
        self.0.get_mut(key)?.downcast_mut()
    }

    pub fn get_result<T: Any, Q: ?Sized + Hash + Eq>(&self, key: &Q) -> Result<&T, GetError>
    where
        K: Borrow<Q>,
    {
        self.0
            .get(key)
            .ok_or(GetError::EmptyKey)?
            .downcast_ref()
            .ok_or(GetError::MismatchedType)
    }
}

use std::fmt::Debug;

// ------------------------------------------------------
// Struct: StateMap
// ------------------------------------------------------

#[derive(Default)]
pub struct StateMap<K>
where
    K: Eq + Hash + Clone + Debug,
{
    map: HashMap<K, Box<dyn IState>>,
}

// ------------------------------------------------------
// Implementation
// ------------------------------------------------------
impl<K> Clone for StateMap<K>
where
    K: Eq + Hash + Clone + Debug,
{
    fn clone(&self) -> Self {
        let mut cloned = HashMap::new();
        for (k, v) in &self.map {
            cloned.insert(k.clone(), v.clone_box());
        }
        Self { map: cloned }
    }
}

impl<K> StateMap<K>
where
    K: Eq + Hash + Clone + Debug,
{
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }

    /// Insert a new IState into the map.
    pub fn insert<T: IState + Default + Clone + 'static>(&mut self, key: K, value: T) {
        self.map.insert(key, Box::new(value));
    }
    pub fn insert_any(&mut self, key: K, value: Box<dyn IState>) {
        self.map.insert(key, value);
    }

    /// Get a reference to a stored type.
    pub fn get<T: IState + 'static, Q: ?Sized + Eq + Hash>(&self, key: &Q) -> Option<&T>
    where
        K: std::borrow::Borrow<Q>,
    {
        self.map.get(key)?.as_ref().as_any()?.downcast_ref()
    }

    /// Get a mutable reference to a stored type.
    pub fn get_mut<T: IState + 'static, Q: ?Sized + Eq + Hash>(&mut self, key: &Q) -> Option<&mut T>
    where
        K: std::borrow::Borrow<Q>,
    {
        self.map.get_mut(key)?.as_mut_any()?.downcast_mut::<T>()
    }

    /// Check if the map contains a given key.
    pub fn contains_key<Q: ?Sized + Eq + Hash>(&self, key: &Q) -> bool
    where
        K: std::borrow::Borrow<Q>,
    {
        self.map.contains_key(key)
    }

    /// Remove a key and return the boxed IState.
    pub fn remove<Q: ?Sized + Eq + Hash>(&mut self, key: &Q) -> Option<Box<dyn IState>>
    where
        K: std::borrow::Borrow<Q>,
    {
        self.map.remove(key)
    }

    /// Clears all entries.
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// Returns the number of entries.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns true if empty.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Iterate over keys and their dyn values.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &Box<dyn IState>)> {
        self.map.iter()
    }

    /// Mutable iterator.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut Box<dyn IState>)> {
        self.map.iter_mut()
    }
}

// ------------------------------------------------------
// Helper trait for downcasting
// ------------------------------------------------------
pub trait AsAny {
    fn as_any(&self) -> Option<&dyn std::any::Any> {
        None
    }
    fn as_mut_any(&mut self) -> Option<&mut dyn std::any::Any> {
        None
    }
}

impl<T: IState + 'static> AsAny for T {
    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
    fn as_mut_any(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }
}

// impl dyn IState {
//     fn as_any(&self) -> Option<&dyn std::any::Any> {
//         None
//     }
//     fn as_mut_any(&mut self) -> Option<&mut dyn std::any::Any> {
//         None
//     }
// }
