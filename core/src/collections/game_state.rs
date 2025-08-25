use std::any::type_name;
use std::hash::Hash;
use std::{any::Any, borrow::Borrow, collections::HashMap};

use egui::mutex::Mutex;
use serde::de::DeserializeOwned;

use crate::dumpster_engine::NetworkModes;
use crate::system::system_game_state::{to_bytes, IState};

pub struct NetworkSynchEvent {
    id: i32,
    payload: Vec<u8>,
}

use serde::{Deserialize, Serialize};

// The "erased" event you actually store in Vec
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub id: i32,
    pub payload: Vec<u8>, // serialized data
}

impl Event {
    // fn new<T: Serialize + 'static>(value: &T) -> Self {
    //     let payload = bincode::serialize(value).unwrap();
    //     Self {
    //         type_id: TypeId::of::<T>(),
    //         type_name: std::any::type_name::<T>(),
    //         payload,
    //     }
    // }

    // fn deserialize<T: for<'de> Deserialize<'de> + 'static>(&self) -> Option<T> {
    //     if self.type_id == TypeId::of::<T>() {
    //         Some(bincode::deserialize(&self.payload).unwrap())
    //     } else {
    //         None
    //     }
    // }
}

pub enum StateOwnerships {
    Instance,
    Host,
}

static mut REGISTERED_GLOBAL_STATES: Option<Mutex<HashMap<i32, CreateFN>>> = None;
static mut REGISTERED_GLOBAL_STATES_SERIALIZE: Option<Mutex<HashMap<i32, SerializerFn>>> = None;
static mut REGISTERED_GLOBAL_STATES_DESERIALIZE: Option<Mutex<HashMap<i32, DeserializerFn>>> = None;

type CreateFN = fn() -> Box<dyn Any>;
type SerializerFn = fn(&dyn Any) -> Vec<u8>;
type DeserializerFn = fn(&[u8]) -> Box<dyn Any>;

pub struct GameState {
    edited_state: Vec<Event>,
    fn_deserialize: HashMap<i32, DeserializerFn>,
    fn_serialize: HashMap<i32, SerializerFn>,
    pub(crate) cache: AnyMap<i32>,
    pub network_mode: NetworkModes,
}
impl GameState {
    pub fn register_global_states<T>()
    where
        T: Any + IState,
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
            let create: fn() -> Box<dyn Any> = || return Box::new(T::default());
            guard.insert(T::id(), create);
        }
        println!("register STATE {}", type_name::<T>());
    }
    pub fn register_global_states_serializable<T>()
    where
        T: IState + Serialize + DeserializeOwned + 'static,
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
            let create: fn() -> Box<dyn Any> = || return Box::new(T::default());
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
                Box::new(obj) as Box<dyn Any>
            };
            guard.insert(T::id(), deserialize);
        }
        println!("register STATE_SERIALIZED {}", type_name::<T>());
    }

    pub fn apply_network_sync_events(&mut self, sync: Vec<Event>) {
        for evnt in sync {
            if !self.fn_deserialize.contains_key(&evnt.id) {
                println!("Unknown id");
                continue;
            }

            let result = self.fn_deserialize[&evnt.id](evnt.payload.as_slice());
            self.cache.insert_any(evnt.id, result);
        }
    }
    pub fn get_network_sync_events(&mut self) -> Vec<Event> {
        let x = self.edited_state.clone();
        self.edited_state.clear();
        return x;
    }
    pub fn change_network_mode(&mut self, mode: NetworkModes) {
        self.network_mode = mode;
    }
    pub fn new(network_mode: NetworkModes) -> GameState {
        let mut cache = AnyMap::<i32>::default();
        let mut fn_serialize = HashMap::<i32, SerializerFn>::default();
        let mut fn_deserialize = HashMap::<i32, DeserializerFn>::default();

        unsafe {
            let Some(unwrapped) = &REGISTERED_GLOBAL_STATES else {
                panic!("Failed to unwrap state");
            };
            let mut guard = unwrapped;
            let z = guard.lock();

            for x in z.iter() {
                cache.insert_any(x.0.clone(), x.1());
            }

            let Some(unwrapped) = &REGISTERED_GLOBAL_STATES_SERIALIZE else {
                panic!("Failed to unwrap state");
            };
            let mut guard = unwrapped;
            let z = guard.lock();

            for x in z.iter() {
                fn_serialize.insert(x.0.clone(), *x.1);
            }

            let Some(unwrapped) = &REGISTERED_GLOBAL_STATES_DESERIALIZE else {
                panic!("Failed to unwrap state");
            };
            let mut guard = unwrapped;
            let z = guard.lock();

            for x in z.iter() {
                fn_deserialize.insert(x.0.clone(), *x.1);
            }
        }
        GameState {
            edited_state: Vec::new(),
            cache: cache,
            network_mode: network_mode,
            fn_deserialize: fn_deserialize,
            fn_serialize: fn_serialize,
        }
    }

    fn has_write_permision(mode: &NetworkModes, ownership: &StateOwnerships) -> bool {
        match mode {
            NetworkModes::Offline => true,
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
                NetworkModes::Offline => false,
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
        if !GameState::has_write_permision(&self.network_mode, &T::ownership()) {
            println!("did not have write permissions for {}", type_name::<T>());
            return;
        }

        let id = T::id();
        let Some(mut val) = self.cache.get_mut::<T, i32>(&id) else {
            // let mut v = self.get_value2::<T>();
            // edit(&mut v);
            // self.set_value2::<T>(v);

            // return;
            panic!("Requested unknown value of type {}", type_name::<T>());
        };

        edit(&mut val);

        if GameState::has_push_permision(&self.network_mode, &T::ownership()) {
            let id2 = T::id();
            let data = self.fn_serialize[&id2](&self.get_value2::<T>());
            self.edited_state.push(Event {
                id: id2.clone(),
                payload: data,
            });
            println!("push type: {}", type_name::<T>());
        }
    }
    fn set_value2<T: 'static>(&mut self, val: T)
    where
        T: IState,
        T: Clone,
    {
        if !GameState::has_write_permision(&self.network_mode, &T::ownership()) {
            println!("did not have write permissions for {}", type_name::<T>());
            return;
        }
        if GameState::has_push_permision(&self.network_mode, &T::ownership()) {
            println!("type push type: {}", type_name::<T>());
        }
        let id = T::id();
        self.cache.insert::<T>(id, val);
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
